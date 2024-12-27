use std::{collections::HashMap, time::Duration};

use futures::{FutureExt, StreamExt};
use log::{debug, info};
use tokio::select;
use tokio_stream::wrappers::BroadcastStream;
use tokio_util::sync::CancellationToken;
use warp::{reply::Reply, Filter};

use crate::{
    config::CONFIG,
    control_api::types::{WsData, WsObject},
    ship,
    sql::{self, DatabaseConnector},
};

use super::types::MyReceiver;

pub struct ControlApiServer {
    context: crate::workers::types::ConductorContext,
    cancellation_token: CancellationToken,
    ship_rx: Option<tokio::sync::broadcast::Receiver<ship::MyShip>>,
    cancellation_tokens: Vec<(String, bool, CancellationToken)>,
}

impl ControlApiServer {
    pub fn new(
        context: crate::workers::types::ConductorContext,
        ship_rx: tokio::sync::broadcast::Receiver<ship::MyShip>,
        cancellation_tokens: Vec<(String, bool, CancellationToken)>,
    ) -> Self {
        Self {
            context,
            cancellation_token: CancellationToken::new(),
            ship_rx: Some(ship_rx),
            cancellation_tokens,
        }
    }

    pub fn new_box(
        context: crate::workers::types::ConductorContext,
        ship_rx: tokio::sync::broadcast::Receiver<ship::MyShip>,
        cancellation_tokens: Vec<(String, bool, CancellationToken)>,
    ) -> Box<Self> {
        Box::new(Self::new(context, ship_rx, cancellation_tokens))
    }

    fn create_broadcast_channels(
        &mut self,
    ) -> (
        tokio::sync::broadcast::Sender<ship::MyShip>,
        MyReceiver<ship::MyShip>,
        MyReceiver<sql::Agent>,
    ) {
        let (ship_broadcast_tx, ship_broadcast_rx) =
            tokio::sync::broadcast::channel::<ship::MyShip>(16);

        if let Some(mut ship_rx) = self.ship_rx.take() {
            let ship_broadcast_tx = ship_broadcast_tx.clone();
            tokio::spawn(async move {
                while let Ok(ship) = ship_rx.recv().await {
                    if let Err(e) = ship_broadcast_tx.send(ship.clone()) {
                        log::error!("Failed to broadcast ship update: {}", e);
                    }
                }
            });
        }

        let agent_rx: MyReceiver<sql::Agent> = MyReceiver(
            self.context
                .database_pool
                .agent_broadcast_channel
                .1
                .resubscribe(),
        );

        let ship_broadcast_rx = MyReceiver(ship_broadcast_rx);
        (ship_broadcast_tx, ship_broadcast_rx, agent_rx)
    }

    fn build_routes(
        &self,
        ship_broadcast_rx: MyReceiver<ship::MyShip>,
        agent_rx: MyReceiver<sql::Agent>,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let main = warp::get()
            .and(warp::path::end())
            .and(warp::fs::file("./index.html"));

        let ws_routes = self.build_websocket_routes(ship_broadcast_rx, agent_rx);
        let ship_route = self.build_ships_route();
        let ship_actions_route = self.build_ship_actions_route();
        let shutdown_route = self.build_shutdown_route();
        let waypoints_route = self.build_waypoints_route();
        let contract_route = self.build_contract_route();
        let contracts_route = self.build_contracts_route();
        let transactions_route = self.build_transactions_route();
        let trade_route_route = self.build_trade_route_route();

        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec![
                "Access-Control-Allow-Origin",
                "Origin",
                "Accept",
                "X-Requested-With",
                "Content-Type",
            ])
            .allow_methods(&[warp::http::Method::GET, warp::http::Method::POST]);

        main.or(ws_routes)
            .or(ship_actions_route)
            .or(ship_route)
            .or(shutdown_route)
            .or(waypoints_route)
            .or(contract_route)
            .or(contracts_route)
            .or(transactions_route)
            .or(trade_route_route)
            .with(cors)
    }

    fn build_websocket_routes(
        &self,
        ship_broadcast_rx: MyReceiver<ship::MyShip>,
        agent_rx: MyReceiver<sql::Agent>,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("ws")
            .and(warp::path("all"))
            .and(warp::ws())
            .and(warp::any().map(move || ship_broadcast_rx.clone()))
            .and(warp::any().map(move || agent_rx.clone()))
            .map(
                |ws: warp::ws::Ws,
                 ship_broadcast_rx: MyReceiver<ship::MyShip>,
                 agent_rx: MyReceiver<sql::Agent>| {
                    ws.on_upgrade(|websocket| {
                        info!("New websocket connection");
                        let (tx, _rx) = websocket.split();
                        let ship_stream = BroadcastStream::new(ship_broadcast_rx.0);
                        let agent_stream = BroadcastStream::new(agent_rx.0);

                        let ship_stream = ship_stream
                            .filter_map(|ship_result| async {
                                ship_result.ok().and_then(|ship| {
                                    serde_json::to_string(&WsObject {
                                        data: WsData::RustShip(ship),
                                    })
                                    .ok()
                                })
                            })
                            .map(|text| Ok(warp::ws::Message::text(text)));

                        let agent_stream = agent_stream
                            .filter_map(|agent_result| async {
                                agent_result.ok().and_then(|agent| {
                                    serde_json::to_string(&WsObject {
                                        data: WsData::MyAgent(agent),
                                    })
                                    .ok()
                                })
                            })
                            .map(|text| Ok(warp::ws::Message::text(text)));

                        let s = futures::stream::select(ship_stream, agent_stream);

                        let forwarded_stream = s.forward(tx).map(|result| {
                            if let Err(e) = result {
                                log::error!("websocket error: {:?}", e);
                            }
                        });

                        forwarded_stream
                    })
                },
            )
    }

    fn build_ships_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let context = self.context.clone();
        warp::path("ships").map(move || {
            debug!("Getting ships");
            let ships = context.ship_manager.get_all_clone();
            debug!("Got {} ships", ships.len());
            warp::reply::json(&ships)
        })
    }

    fn build_ship_actions_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let context = self.context.clone();
        warp::path!("ship" / String / "navigate")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |symbol: String, body: serde_json::Value| {
                debug!("Navigating to waypoint for ship {}", symbol);
                let ship_clone = context.ship_manager.get_clone(&symbol);
                let waypoint_id = body["waypointSymbol"].as_str();

                if waypoint_id.is_none() {
                    return warp::reply::with_status(
                        warp::reply::json(&"Waypoint not found"),
                        warp::http::StatusCode::BAD_REQUEST,
                    );
                }

                if let Some(ship) = ship_clone {
                    if ship.role != ship::Role::Manuel {
                        return warp::reply::with_status(
                            warp::reply::json(&"Ship not in Manuel mode"),
                            warp::http::StatusCode::BAD_REQUEST,
                        );
                    }
                } else {
                    return warp::reply::with_status(
                        warp::reply::json(&"Ship not found"),
                        warp::http::StatusCode::NOT_FOUND,
                    );
                }

                let mut resp = HashMap::new();
                resp.insert(
                    "waypointSymbol".to_string(),
                    waypoint_id.unwrap().to_string(),
                );
                resp.insert("shipSymbol".to_string(), symbol.to_string());

                let context = context.clone();
                let waypoint_id = waypoint_id.unwrap().to_string();

                tokio::spawn(async move {
                    let mut ship_guard = context.ship_manager.get_mut(&symbol).await;
                    let ship = ship_guard.value_mut().unwrap();
                    let waypoints = {
                        context
                            .all_waypoints
                            .get(&ship.nav.system_symbol)
                            .unwrap()
                            .clone()
                    };
                    ship.nav_to(
                        &waypoint_id,
                        true,
                        &waypoints,
                        &context.api,
                        context.database_pool.clone(),
                        sql::TransactionReason::None,
                    )
                    .await
                    .unwrap();
                });

                resp.insert("success".to_string(), "true".to_string());

                warp::reply::with_status(warp::reply::json(&resp), warp::http::StatusCode::OK)
            })
    }

    fn build_trade_route_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let context = self.context.clone();
        warp::path("tradeRoutes")
            .and_then(move || Self::trade_routes_route_handler(context.clone()))
    }

    async fn trade_routes_route_handler(
        context: crate::workers::types::ConductorContext,
    ) -> Result<impl Reply, warp::Rejection> {
        let trade_routes = sql::TradeRoute::get_summarys(&context.database_pool)
            .await
            .unwrap();
        Ok(warp::reply::json(&trade_routes))
    }

    fn build_contract_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let context = self.context.clone();
        warp::path!("contracts" / String)
            .and_then(move |id: String| Self::contract_route_handler(context.clone(), id))
    }

    async fn contract_route_handler(
        context: crate::workers::types::ConductorContext,
        id: String,
    ) -> Result<impl Reply, warp::Rejection> {
        let contract: sql::Contract = sql::Contract::get_by_id(&context.database_pool, &id)
            .await
            .unwrap();

        let deliveries: Vec<sql::ContractDelivery> =
            sql::ContractDelivery::get_by_contract_id(&context.database_pool, &id)
                .await
                .unwrap();

        let transactions: Vec<sql::MarketTransaction> = sql::MarketTransaction::get_by_reason(
            &context.database_pool,
            sql::TransactionReason::Contract(id.clone()),
        )
        .await
        .unwrap();

        Ok(warp::reply::json(&(id, contract, deliveries, transactions)))
    }

    fn build_contracts_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let context = self.context.clone();
        warp::path("contracts").and_then(move || Self::contracts_route_handler(context.clone()))
    }

    async fn contracts_route_handler(
        context: crate::workers::types::ConductorContext,
    ) -> Result<impl Reply, warp::Rejection> {
        debug!("Getting contracts");
        let contracts = sql::Contract::get_all_sm(&context.database_pool)
            .await
            .unwrap();
        debug!("Got {} contracts", contracts.len());
        Ok(warp::reply::json(&contracts))
    }

    fn build_transactions_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let context = self.context.clone();
        warp::path("transactions")
            .and_then(move || Self::transactions_route_handler(context.clone()))
    }

    async fn transactions_route_handler(
        context: crate::workers::types::ConductorContext,
    ) -> Result<impl Reply, warp::Rejection> {
        debug!("Getting transactions");
        let transactions = sql::MarketTransaction::get_all(&context.database_pool)
            .await
            .unwrap();
        debug!("Got {} transactions", transactions.len());
        Ok(warp::reply::json(&transactions))
    }

    fn build_waypoints_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let context = self.context.clone();

        warp::path("waypoints").map(move || {
            debug!("Getting waypoints");
            let waypoints: std::collections::HashMap<
                String,
                std::collections::HashMap<String, space_traders_client::models::Waypoint>,
            > = {
                context
                    .all_waypoints
                    .iter()
                    .map(|w| w.clone())
                    .map(|f| (f.values().next().unwrap().system_symbol.clone(), f))
                    .collect()
            };

            debug!("Got {} waypoints", waypoints.len());
            warp::reply::json(&waypoints)
            // warp::reply::reply()
        })
    }

    fn build_shutdown_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let cancel_tokens = self.cancellation_tokens.clone();
        warp::path("shutdown").and(warp::post()).map(move || {
            info!("Shutting down server");
            cancel_tokens.iter().for_each(|(_, independent, token)| {
                if *independent {
                    token.cancel();
                }
            });
            warp::reply::reply()
        })
    }

    async fn run_server(&mut self) -> anyhow::Result<()> {
        info!("Starting control api server");

        if !CONFIG.control_server.active {
            info!("Control api not active, exiting");
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(
            CONFIG.control_server.start_sleep_duration,
        ))
        .await;

        let (_, ship_broadcast_rx, agent_rx) = self.create_broadcast_channels();
        let routes = self.build_routes(ship_broadcast_rx, agent_rx);

        select! {
            _ = self.cancellation_token.cancelled() => {
                info!("Shutting down server via cancellation");
            },
            _ = warp::serve(routes).run(CONFIG.control_server.socket_address).fuse() => {
                info!("Shutting down server");
            }
        }

        info!("Control api server done");
        Ok(())
    }
}

impl crate::workers::types::Conductor for ControlApiServer {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_server().await })
    }

    fn get_name(&self) -> String {
        "ControlApiServer".to_string()
    }

    fn get_cancel_token(&self) -> tokio_util::sync::CancellationToken {
        self.cancellation_token.clone()
    }

    fn is_independent(&self) -> bool {
        false
    }
}
