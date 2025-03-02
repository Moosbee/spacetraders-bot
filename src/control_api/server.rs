use std::time::Duration;

use futures::FutureExt;
use tokio_util::sync::CancellationToken;

use crate::{config::CONFIG, manager::Manager, ship, sql, types::ConductorContext};

use super::types::MyReceiver;

pub struct ControlApiServer {
    context: ConductorContext,
    cancellation_token: CancellationToken,
    ship_rx: Option<tokio::sync::broadcast::Receiver<ship::MyShip>>,
    ship_cancellation_token: CancellationToken,
}

impl ControlApiServer {
    pub fn new(
        context: ConductorContext,
        ship_rx: tokio::sync::broadcast::Receiver<ship::MyShip>,
        cancellation_token: CancellationToken,
        ship_cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            context,
            cancellation_token,
            ship_rx: Some(ship_rx),
            ship_cancellation_token,
        }
    }

    pub fn new_box(
        context: ConductorContext,
        ship_rx: tokio::sync::broadcast::Receiver<ship::MyShip>,
        _cancellation_tokens: Vec<(String, bool, CancellationToken)>,
    ) -> Box<Self> {
        Box::new(Self::new(
            context,
            ship_rx,
            CancellationToken::new(),
            CancellationToken::new(),
        ))
    }

    fn setup_broadcast_channels(
        &mut self,
    ) -> anyhow::Result<(MyReceiver<ship::MyShip>, MyReceiver<sql::Agent>)> {
        let (ship_tx, ship_rx) = tokio::sync::broadcast::channel(16);

        if let Some(mut incoming_ship_rx) = self.ship_rx.take() {
            let ship_tx = ship_tx.clone();
            tokio::spawn(async move {
                while let Ok(ship) = incoming_ship_rx.recv().await {
                    if let Err(e) = ship_tx.send(ship.clone()) {
                        log::error!("Failed to broadcast ship update: {}", e);
                    }
                }
            });
        }

        let agent_rx = MyReceiver(
            self.context
                .database_pool
                .agent_broadcast_channel
                .1
                .resubscribe(),
        );

        Ok((MyReceiver(ship_rx), agent_rx))
    }

    async fn run_server(&mut self) -> anyhow::Result<()> {
        if !CONFIG.control_server.active {
            log::info!("Control API not active, exiting");
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(
            CONFIG.control_server.start_sleep_duration,
        ))
        .await;

        let (ship_rx, agent_rx) = self.setup_broadcast_channels()?;
        let routes = crate::control_api::routes::build_routes(
            self.context.clone(),
            ship_rx,
            agent_rx,
            self.ship_cancellation_token.clone(),
        );

        tokio::select! {
            _ = self.cancellation_token.cancelled() => {
                log::info!("Shutting down server via cancellation");
            },
            _ = warp::serve(routes).run(CONFIG.control_server.socket_address).fuse() => {
                log::info!("Server shutdown completed");
            }
        }

        Ok(())
    }
}

// Implement the Conductor trait
impl crate::workers::types::Conductor for ControlApiServer {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_server().await })
    }

    fn get_name(&self) -> String {
        "ControlApiServer".to_string()
    }

    fn get_cancel_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    fn is_independent(&self) -> bool {
        false
    }
}

impl Manager for ControlApiServer {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::error::Result<()>> + Send + '_>>
    {
        Box::pin(async move {
            self.run_server()
                .await
                .map_err(|e| crate::error::Error::General(e.to_string()))
        })
    }

    fn get_name(&self) -> &str {
        "ControlApiServer"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancellation_token
    }
}
