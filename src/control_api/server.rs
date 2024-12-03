use std::time::Duration;

use futures::{FutureExt, StreamExt};
use log::{debug, info};
use tokio::select;
use tokio_stream::wrappers::BroadcastStream;
use tokio_util::sync::CancellationToken;
use warp::Filter;

use crate::{
    config::CONFIG,
    control_api::types::{WsData, WsObject},
    ship, sql,
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
        let shutdown_route = self.build_shutdown_route();
        let waypoints_route = self.build_waypoints_route();

        let cors = warp::cors().allow_any_origin();

        main.or(ws_routes)
            .or(ship_route)
            .or(shutdown_route)
            .or(waypoints_route)
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
            let ships = context.ship_manager.get_ships_clone();
            debug!("Got {} ships", ships.len());
            warp::reply::json(&ships)
        })
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
