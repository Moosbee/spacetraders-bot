use std::collections::HashMap;

use database::{DatabaseConnector, ShipInfo};
use space_traders_client::models;
use tracing::debug;
use utils::get_system_symbol;

use crate::{error::Result, manager::Manager, utils::ConductorContext};

use super::{message::FleetManagerMessage, messanger::FleetManagerMessanger};

pub struct FleetManager {
    cancel_token: tokio_util::sync::CancellationToken,
    receiver: tokio::sync::mpsc::Receiver<FleetManagerMessage>,
    context: ConductorContext,
}

impl FleetManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<FleetManagerMessage>,
        FleetManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(24);
        debug!("Created FleetManager channel");

        (receiver, FleetManagerMessanger::new(sender))
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<FleetManagerMessage>,
    ) -> Self {
        debug!("Creating new FleetManager");
        Self {
            cancel_token,
            context,
            receiver,
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::fleet_manager_worker",
        skip(self),
        err(Debug)
    )]
    async fn run_fleet_worker(&mut self) -> std::result::Result<(), crate::error::Error> {
        while !self.cancel_token.is_cancelled() {
            let message = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.cancel_token.cancelled() => None
            };
            debug!("Received FleetManager message: {:?}", message);

            match message {
                Some(message) => {
                    self.handle_fleet_message(message).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::handle_fleet_message",
        skip(self),
        err(Debug)
    )]
    async fn handle_fleet_message(&mut self, message: super::message::FleetMessage) -> Result<()> {
        match message {
            super::message::FleetMessage::ScrapperAtShipyard {
                waypoint_symbol,
                ship_symbol,
                callback,
            } => {
                let erg = self
                    .handle_scrapper_at_shipyard(&waypoint_symbol, &ship_symbol)
                    .await;
                callback.send(ship_symbol).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?;
                erg?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::fleet_manager::handle_scrapper_at_shipyard",
        skip(self)
    )]
    async fn handle_scrapper_at_shipyard(
        &mut self,
        waypoint_symbol: &str,
        _ship_symbol: &str,
    ) -> Result<()> {
        return Ok(());
    }
}

impl Manager for FleetManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_fleet_worker().await })
    }

    fn get_name(&self) -> &str {
        "FleetManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
