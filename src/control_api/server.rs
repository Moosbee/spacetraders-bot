use std::time::Duration;

use futures::{FutureExt, StreamExt};
use log::{debug, info};
use tokio::select;
use tokio_stream::wrappers::BroadcastStream;
use tokio_util::sync::CancellationToken;
use warp::Filter;

use crate::{config::CONFIG, ship};

pub struct ControlApiServer {
    context: crate::workers::types::ConductorContext,
    cancellation_token: CancellationToken,
    ship_rx: Option<tokio::sync::mpsc::Receiver<ship::MyShip>>,
    cancellation_tokens: Vec<(String, bool, CancellationToken)>,
}

impl ControlApiServer {
    pub fn new(
        context: crate::workers::types::ConductorContext,
        ship_rx: tokio::sync::mpsc::Receiver<ship::MyShip>,
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
        ship_rx: tokio::sync::mpsc::Receiver<ship::MyShip>,
        cancellation_tokens: Vec<(String, bool, CancellationToken)>,
    ) -> Box<Self> {
        Box::new(Self::new(context, ship_rx, cancellation_tokens))
    }

    fn create_ship_broadcast_channel(
        &mut self,
    ) -> (tokio::sync::broadcast::Sender<ship::MyShip>, MyReceiver) {
        let (ship_broadcast_tx, ship_broadcast_rx) =
            tokio::sync::broadcast::channel::<ship::MyShip>(16);

        if let Some(mut ship_rx) = self.ship_rx.take() {
            let ship_broadcast_tx = ship_broadcast_tx.clone();
            tokio::spawn(async move {
                while let Some(ship) = ship_rx.recv().await {
                    if let Err(e) = ship_broadcast_tx.send(ship.clone()) {
                        log::error!("Failed to broadcast ship update: {}", e);
                    }
                }
            });
        }

        let ship_broadcast_rx = MyReceiver(ship_broadcast_rx);
        (ship_broadcast_tx, ship_broadcast_rx)
    }

    fn build_routes(
        &self,
        ship_broadcast_rx: MyReceiver,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let main = warp::get()
            .and(warp::path::end())
            .and(warp::fs::file("./index.html"));

        let ws_routes = self.build_websocket_routes(ship_broadcast_rx.clone());
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
        ship_broadcast_rx: MyReceiver,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("ws")
            .and(warp::path("ships"))
            .and(warp::ws())
            .and(warp::any().map(move || ship_broadcast_rx.clone()))
            .map(|ws: warp::ws::Ws, ship_broadcast_rx: MyReceiver| {
                ws.on_upgrade(|websocket| {
                    info!("New websocket connection");
                    let (tx, _rx) = websocket.split();
                    let ship_stream = BroadcastStream::new(ship_broadcast_rx.0);

                    let forwarded_stream = ship_stream
                        .filter_map(|ship_result| async {
                            ship_result
                                .ok()
                                .and_then(|ship| serde_json::to_string(&ship).ok())
                        })
                        .map(|text| Ok(warp::ws::Message::text(text)))
                        .forward(tx)
                        .map(|result| {
                            if let Err(e) = result {
                                log::error!("websocket error: {:?}", e);
                            }
                        });

                    forwarded_stream
                })
            })
    }

    fn build_ships_route(
        &self,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let context = self.context.clone();
        warp::path("ships").map(move || {
            debug!("Getting ships");
            let ships = crate::control_api::unsafe_clone(&context.my_ships);
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

        let (_, ship_broadcast_rx) = self.create_ship_broadcast_channel();
        let routes = self.build_routes(ship_broadcast_rx);

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

struct MyReceiver(tokio::sync::broadcast::Receiver<ship::MyShip>);

impl Clone for MyReceiver {
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }

    fn clone(&self) -> Self {
        MyReceiver(self.0.resubscribe())
    }
}
