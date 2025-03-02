use std::sync::Arc;

use log::{debug, info};
use space_traders_client::models::{self};

use crate::{
    config::CONFIG,
    error::Result,
    manager::{mining_manager::mining_messages::MiningMessage, Manager},
    ship,
    types::ConductorContext,
};

use super::{
    mining_manager_messanger::MiningManagerMessanger,
    mining_messages::{AssignWaypointMessage, ExtractionNotification, MiningManagerMessage},
    place_finder::ActionType,
    ship_inventory_manager::ShipInventoryManager,
    transfer_manager::TransferManager,
    waypoint_manager::WaypointManager,
};

#[derive(Debug)]
pub struct MiningManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<MiningManagerMessage>,
    transfer_manager: Arc<TransferManager>,
    inventory_manager: ShipInventoryManager,
    waypoint_manager: WaypointManager,
}

impl MiningManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<MiningManagerMessage>,
        MiningManagerMessanger,
        Arc<TransferManager>,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
        debug!("Created MiningManager channel");
        let transfer_manager = Arc::new(TransferManager::new());

        (
            receiver,
            MiningManagerMessanger::new(sender, transfer_manager.clone()),
            transfer_manager,
        )
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<MiningManagerMessage>,
        transfer_manager: Arc<TransferManager>,
    ) -> Self {
        debug!("Initializing new MiningManager");
        Self {
            cancel_token,
            receiver,
            transfer_manager,
            inventory_manager: ShipInventoryManager::new(),
            waypoint_manager: WaypointManager::new(
                context.clone(),
                CONFIG.mining.max_miners_per_waypoint,
            ),
            context,
        }
    }

    async fn run_mining_worker(&mut self) -> Result<()> {
        debug!("Starting MiningManager worker");
        while !self.cancel_token.is_cancelled() {
            let message: Option<MiningMessage> = tokio::select! {
                message = self.receiver.recv() => message,
                _ = self.cancel_token.cancelled() => None
            };
            match message {
                Some(message) => {
                    debug!("Handling message: {}", message);
                    self.handle_message(message).await?;
                }
                None => {
                    debug!("No more messages, exiting loop");
                    break;
                }
            }
        }

        debug!("MiningManager worker stopped");
        Ok(())
    }

    async fn handle_message(&mut self, message: MiningManagerMessage) -> Result<()> {
        match message {
            MiningMessage::AssignWaypoint(message) => {
                self.handle_assign_waypoint_message(message).await?;
            }
            MiningMessage::ExtractionNotification(message) => {
                self.handle_extraction_notification(message).await?;
            }
        }
        Ok(())
    }

    async fn handle_assign_waypoint_message(
        &mut self,
        message: AssignWaypointMessage,
    ) -> Result<()> {
        match message {
            AssignWaypointMessage::AssignWaypoint {
                ship_clone,
                callback,
                is_syphon,
            } => {
                debug!("Assigning waypoint for ship: {}", ship_clone.symbol);
                let erg = self
                    .waypoint_manager
                    .assign_waypoint_syphon(ship_clone, is_syphon)
                    .await;
                debug!(
                    "Waypoint assignment result: {:?} is_syphon: {}",
                    erg, is_syphon
                );
                let _send = callback.send(erg);
            }
            AssignWaypointMessage::NotifyWaypoint {
                ship_clone,
                callback,
            } => {
                debug!("Notifying waypoint for ship: {}", ship_clone.symbol);
                let erg = self.waypoint_manager.notify_waypoint(ship_clone).await;
                let _send = callback.send(erg);
            }
            AssignWaypointMessage::UnassignWaypoint {
                ship_clone,
                callback,
            } => {
                debug!("Unassigning waypoint for ship: {}", ship_clone.symbol);
                let erg = self.waypoint_manager.unassign_waypoint(ship_clone).await;
                let _send = callback.send(erg);
            }
            AssignWaypointMessage::UnassignWaypointComplete {
                ship_clone,
                callback,
            } => {
                debug!("Waypoint unassignment complete for ship: {:?}", ship_clone);
                let erg = self
                    .waypoint_manager
                    .unassign_waypoint_complete(ship_clone)
                    .await;
                let _send = callback.send(erg);
            }
        }

        Ok(())
    }

    async fn handle_extraction_notification(
        &mut self,
        message: ExtractionNotification,
    ) -> Result<()> {
        match message {
            ExtractionNotification::GetNextWaypoint {
                ship_clone,
                callback,
            } => {
                debug!("Getting next waypoint for ship: {}", ship_clone.symbol);
                let erg = self.get_next_waypoint(ship_clone).await;
                let _send = callback.send(erg);
            }
            ExtractionNotification::ExtractionComplete { ship, waypoint } => {
                debug!(
                    "Extraction complete for ship: {:?} at waypoint: {:?}",
                    ship, waypoint
                );
                let _erg = self.process_possible_transfers(&waypoint).await?;
            }
            ExtractionNotification::TransportArrived { ship, waypoint } => {
                debug!(
                    "Transport arrived for ship: {:?} at waypoint: {:?}",
                    ship, waypoint
                );
                let _erg = self.process_possible_transfers(&waypoint).await?;
            }
        }
        Ok(())
    }

    async fn process_possible_transfers(&mut self, waypoint_symbol: &str) -> Result<()> {
        debug!(
            "Processing possible transfers at waypoint: {:?}",
            waypoint_symbol
        );
        let mut current_trade_symbol = None;

        let mut count = 10;
        loop {
            if count <= 0 {
                break;
            }
            let ships = self.get_ships_at_waypoint(waypoint_symbol).await?;
            let (extraction_ships, transport_ships) = self.partition_ships_by_role(ships);

            debug!(
                "Found {} extraction ships and {} transport ships at waypoint: {:?}",
                extraction_ships.len(),
                transport_ships.len(),
                waypoint_symbol
            );

            let extraction_ships = self.filter_ships_with_cargo(
                extraction_ships
                    .into_iter()
                    .filter(|f| self.transfer_manager.valid_extractor(&f.symbol))
                    .collect::<Vec<_>>(),
            );
            if extraction_ships.is_empty() {
                debug!("No ships with cargo at waypoint: {:?}", waypoint_symbol);
                return Ok(());
            }

            let transport_ships = self.filter_ships_with_space(
                transport_ships
                    .into_iter()
                    .filter(|f| self.transfer_manager.valid_transporter(&f.symbol))
                    .collect::<Vec<_>>(),
            );
            if transport_ships.is_empty() {
                debug!(
                    "No transport ships with space at waypoint: {:?}",
                    waypoint_symbol
                );
                return Ok(());
            }

            let trade_symbol = match current_trade_symbol {
                Some(symbol) => symbol,
                None => {
                    let symbol = self
                        .inventory_manager
                        .determine_most_abundant_cargo(&extraction_ships);
                    if symbol.is_none() {
                        debug!(
                            "No trade symbol determined at waypoint: {:?}",
                            waypoint_symbol
                        );
                        return Ok(());
                    }
                    symbol
                }
            };
            current_trade_symbol = Some(trade_symbol.clone());

            debug!("Executing transfer with trade symbol: {:?}", trade_symbol);
            let transfer_result = self
                .execute_transfer(
                    &transport_ships,
                    &extraction_ships,
                    trade_symbol.as_ref().unwrap(),
                )
                .await?;

            if transfer_result {
                self.waypoint_manager.up_date(waypoint_symbol);
            } else {
                debug!("Transfer failed, resetting trade symbol");
                count = count - 1;
                current_trade_symbol = None;
            }
        }

        Ok(())
    }

    async fn get_ships_at_waypoint(&self, waypoint_symbol: &str) -> Result<Vec<ship::MyShip>> {
        debug!("Fetching ships at waypoint: {:?}", waypoint_symbol);
        Ok(self
            .context
            .ship_manager
            .get_all_clone()
            .await
            .into_iter()
            .filter(|f| f.1.nav.waypoint_symbol == waypoint_symbol)
            .filter(|f| !f.1.nav.is_in_transit())
            .filter(|f| f.1.role == crate::sql::ShipInfoRole::Mining)
            .map(|f| f.1)
            .collect())
    }

    fn partition_ships_by_role(
        &self,
        ships: Vec<ship::MyShip>,
    ) -> (Vec<ship::MyShip>, Vec<ship::MyShip>) {
        debug!("Partitioning ships by role");
        ships.into_iter().partition(|f| {
            let action = ActionType::get_action(&f).is_some();
            action
        })
    }

    /// Filters the provided list of ships, returning only those that have cargo units on board.
    ///
    /// # Arguments
    ///
    /// * `ships` - A vector of `MyShip` instances to be filtered.
    ///
    /// # Returns
    ///
    /// A vector containing only the ships that have cargo units greater than zero.
    fn filter_ships_with_cargo(&self, ships: Vec<ship::MyShip>) -> Vec<ship::MyShip> {
        debug!("Filtering ships with cargo");
        ships.into_iter().filter(|f| f.cargo.units > 0).collect()
    }

    fn filter_ships_with_space(&self, ships: Vec<ship::MyShip>) -> Vec<ship::MyShip> {
        debug!("Filtering ships with space");
        ships
            .into_iter()
            .filter(|f| f.cargo.units < f.cargo.capacity)
            .collect()
    }

    async fn execute_transfer(
        &mut self,
        transport_ships: &[ship::MyShip],
        extraction_ships: &[ship::MyShip],
        trade_symbol: &models::TradeSymbol,
    ) -> Result<bool> {
        debug!("Executing transfer for trade symbol: {:?}", trade_symbol);
        let transporter = match self
            .inventory_manager
            .find_best_transporter(transport_ships, trade_symbol)
        {
            Some(t) => t,
            None => {
                debug!("No suitable transporter found");
                return Ok(false);
            }
        };

        let extractor = match self
            .inventory_manager
            .find_best_extractor(extraction_ships, trade_symbol)
        {
            Some(e) => e,
            None => {
                debug!("No suitable extractor found");
                return Ok(false);
            }
        };

        let transfer_amount = std::cmp::min(extractor.amount, transporter.available_space);
        if transfer_amount > 0
            && self
                .transfer_manager
                .viable(&extractor.ship_symbol, &transporter.ship_symbol)
        {
            info!(
                "Processing transfer: {} of {} from {:?} to {:?}",
                transfer_amount, trade_symbol, extractor.ship_symbol, transporter.ship_symbol
            );
            let erg = self
                .transfer_manager
                .process_transfer(
                    &extractor.ship_symbol,
                    &transporter.ship_symbol,
                    *trade_symbol,
                    transfer_amount,
                )
                .await;

            match erg {
                Ok(_) => {}
                Err(err) => match err {
                    crate::manager::mining_manager::transfer_manager::Error::TransporterDropped { symbol, from, to } => log::error!(
                        "Transporter dropped: {} from {} to {}",
                        symbol,
                        from,
                        to
                    ),
                    crate::manager::mining_manager::transfer_manager::Error::ExtractorDropped { symbol, from, to } => log::error!(
                        "Extractor dropped: {} from {} to {}",
                        symbol,
                        from,
                        to
                    ),
                },
            }

            Ok(true)
        } else {
            debug!(
                "Transfer amount is zero, skipping {:?} to {:?} with {} {:?}",
                extractor.ship_symbol,
                transporter.ship_symbol,
                transfer_amount,
                self.transfer_manager
                    .viable(&extractor.ship_symbol, &transporter.ship_symbol)
            );

            Ok(false)
        }
    }

    async fn get_next_waypoint(&self, ship_clone: crate::ship::MyShip) -> Result<String> {
        debug!("Getting next waypoint for ship: {}", ship_clone.symbol);
        let the_ships: std::collections::HashMap<String, ship::MyShip> =
            self.context.ship_manager.get_all_clone().await;
        let route = self
            .waypoint_manager
            .calculate_waypoint_urgencys(&the_ships);

        debug!("Calculated routes: {:?}", route);
        let routes = route.into_iter().filter(|r| r.1 > 0).collect::<Vec<_>>();

        if routes.is_empty() {
            info!("No routes found for {}", ship_clone.symbol);
            return Err("No routes found".into());
        }

        let route = routes.last().unwrap();
        debug!("Selected route: {:?}", route);

        Ok(route.0.clone())
    }
}

impl Manager for MiningManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_mining_worker().await })
    }

    fn get_name(&self) -> &str {
        "MiningManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
