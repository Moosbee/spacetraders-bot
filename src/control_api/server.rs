use std::time::Duration;

use futures::{FutureExt, StreamExt};
use log::{debug, info};
use tokio::select;
use tokio_util::sync::CancellationToken;
use warp::Filter;

use crate::config::CONFIG;

pub struct ControlApiServer {
    context: crate::workers::types::ConductorContext,
    cancellation_token: CancellationToken,
}

impl ControlApiServer {
    pub fn new_box(_context: crate::workers::types::ConductorContext) -> Box<Self> {
        Box::new(ControlApiServer {
            context: _context,
            cancellation_token: CancellationToken::new(),
        })
    }

    async fn run_server(&self) -> anyhow::Result<()> {
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

        let ws_routes = warp::path("ws")
            // The `ws()` filter will prepare the Websocket handshake.
            .and(warp::ws())
            .map(|ws: warp::ws::Ws| {
                // And then our closure will be called when it completes...
                ws.on_upgrade(|websocket| {
                    // Just echo all messages back...

                    let (tx, rx) = websocket.split();

                    rx.map(|msg| {
                        info!("Received message: {:?}", msg);
                        msg
                    })
                    .forward(tx)
                    .map(|result| {
                        if let Err(e) = result {
                            eprintln!("websocket error: {:?}", e);
                        }
                    })
                })
            });
        let cn_tx = self.context.clone();
        let routes = readme.or(ws_routes).or(warp::path("ships").map(move || {
            debug!("Getting ships");
            // let ships_si = (*cn_tx.my_ships).into_read_only();
            // let ships = ships_si.iter().map(|s| (s.1).clone()).collect::<Vec<_>>();
            let ships = crate::control_api::unsafe_clone(&cn_tx.my_ships);
            debug!("Got {} ships", ships.len());

            warp::reply::json(&ships)
        }));

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
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_server().await })
    }

    fn get_name(&self) -> String {
        "ControlApiServer".to_string()
    }

    fn get_cancel_token(&self) -> tokio_util::sync::CancellationToken {
        self.cancellation_token.clone()
    }
}
