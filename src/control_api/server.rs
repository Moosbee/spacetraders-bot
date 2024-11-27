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
    cancellation_tokens: Vec<(String, bool, tokio_util::sync::CancellationToken)>,
}

impl ControlApiServer {
    pub fn new_box(
        _context: crate::workers::types::ConductorContext,
        ship_rx: tokio::sync::mpsc::Receiver<ship::MyShip>,
        cancellation_tokens: Vec<(String, bool, tokio_util::sync::CancellationToken)>,
    ) -> Box<Self> {
        Box::new(ControlApiServer {
            context: _context,
            cancellation_token: CancellationToken::new(),
            ship_rx: Some(ship_rx),
            cancellation_tokens,
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

        let readme = warp::get()
            .and(warp::path::end())
            .and(warp::fs::file("./index.html"));

        // Match any request and return hello world!
        // let routes = warp::any().map(|| "Hello, World!");

        let (ship_broadcast_tx, ship_broadcast_rx) =
            tokio::sync::broadcast::channel::<ship::MyShip>(16);

        if let Some(mut ship_rx) = self.ship_rx.take() {
            tokio::spawn(async move {
                while let Some(ship) = ship_rx.recv().await {
                    ship_broadcast_tx.send(ship.clone()).unwrap();
                }
            });
        }

        let ship_broadcast_rx = MyReceiver(ship_broadcast_rx);

        let ship_broadcast_rx = warp::any().map(move || ship_broadcast_rx.clone());

        let ws_routes = warp::path("ws")
            .and(warp::path("ships"))
            // The `ws()` filter will prepare the Websocket handshake.
            .and(warp::ws())
            .and(ship_broadcast_rx)
            .map(|ws: warp::ws::Ws, ship_broadcast_rx: MyReceiver| {
                // And then our closure will be called when it completes...
                ws.on_upgrade(|websocket| {
                    // Just echo all messages back...

                    let (tx, _rx) = websocket.split();

                    let ship_stream = BroadcastStream::new(ship_broadcast_rx.0);
                    async fn fun_name(
                        ship: Result<
                            ship::MyShip,
                            tokio_stream::wrappers::errors::BroadcastStreamRecvError,
                        >,
                    ) -> Option<String> {
                        let erg = match ship {
                            Ok(ship) => {
                                let json = serde_json::to_string(&ship);
                                let erg = match json {
                                    Ok(json) => Some(json),
                                    Err(e) => None,
                                };
                                erg
                            }
                            Err(e) => None,
                        };
                        erg
                    }

                    let erg = ship_stream
                        .filter_map(fun_name)
                        .map(|text| Ok(warp::ws::Message::text(text)))
                        .forward(tx)
                        .map(|result| {
                            if let Err(e) = result {
                                eprintln!("websocket error: {:?}", e);
                            }
                        });

                    erg
                })
            });
        let cn_tx = self.context.clone();
        let ship_route = warp::path("ships").map(move || {
            debug!("Getting ships");
            // let ships_si = (*cn_tx.my_ships).into_read_only();
            // let ships = ships_si.iter().map(|s| (s.1).clone()).collect::<Vec<_>>();
            let ships = crate::control_api::unsafe_clone(&cn_tx.my_ships);
            debug!("Got {} ships", ships.len());

            warp::reply::json(&ships)
        });
        let cancel_tokens = self.cancellation_tokens.clone();

        let shutdown = warp::path("shutdown").and(warp::post()).map(move || {
            cancel_tokens.iter().for_each(|(_, independent, token)| {
                if *independent {
                    token.cancel();
                }
            });

            warp::reply::reply()
        });
        let routes = readme.or(ws_routes).or(ship_route).or(shutdown);

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

    // fn get_routes(&self) -> impl Filter... {}
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
