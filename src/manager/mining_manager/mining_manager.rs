use log::{debug, info};
use space_traders_client::models::{self};

use crate::{
    config::CONFIG,
    error::Result,
    manager::{mining_manager::mining_messages::MiningMessage, Manager},
    ship,
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
    context: crate::workers::types::ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<MiningManagerMessage>,
    transfer_manager: TransferManager,
    inventory_manager: ShipInventoryManager,
    waypoint_manager: WaypointManager,
}

impl MiningManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<MiningManagerMessage>,
        MiningManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, MiningManagerMessanger { sender })
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: crate::workers::types::ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<MiningManagerMessage>,
    ) -> Self {
        Self {
            cancel_token,
            receiver,
            transfer_manager: TransferManager::new(),
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
            let message: Option<MiningMessage> = self.receiver.recv().await;
            debug!("Received mining message: {:?}", message);
            match message {
                Some(message) => {
                    self.handle_message(message).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    async fn handle_message(&mut self, message: MiningManagerMessage) -> Result<()> {
        match message {
            MiningMessage::AssignWaypoint(message) => {
                self.handle_assign_waypoint_message(message).await?
            }
            MiningMessage::ExtractionNotification(message) => {
                self.handle_extraction_notification(message).await?
            }
        }
        Ok(())
    }

    async fn handle_assign_waypoint_message(
        &mut self,
        message: AssignWaypointMessage,
    ) -> Result<()> {
        match message {
            // Handle assigning a waypoint to a ship
            AssignWaypointMessage::AssignWaypoint {
                ship_clone,
                callback,
                is_syphon,
            } => {
                let erg = self
                    .waypoint_manager
                    .assign_waypoint_syphon(ship_clone, is_syphon)
                    .await;
                let _send = callback.send(erg);
            }
            // Handle notification that a ship has arrived at a waypoint
            AssignWaypointMessage::NotifyWaypoint {
                ship_clone,
                callback,
            } => {
                let erg = self.waypoint_manager.notify_waypoint(ship_clone).await;
                let _send = callback.send(erg);
            }
            // Handle unassigning a ship from a waypoint
            AssignWaypointMessage::UnassignWaypoint {
                ship_clone,
                callback,
            } => {
                let erg = self.waypoint_manager.unassign_waypoint(ship_clone).await;
                let _send = callback.send(erg);
            }
            // Handle completion of unassigning a ship
            AssignWaypointMessage::UnassignWaypointComplete {
                ship_clone,
                callback,
            } => {
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
                let erg = self.get_next_waypoint(ship_clone).await;

                let _send = callback.send(erg);
            }
            ExtractionNotification::ExtractionComplete { ship: _, waypoint } => {
                let _erg = self.process_possible_transfers(&waypoint).await?;
            }
            ExtractionNotification::TransportArrived { ship: _, waypoint } => {
                let _erg = self.process_possible_transfers(&waypoint).await?;
            }
        }
        Ok(())
    }

    async fn process_possible_transfers(&mut self, waypoint_symbol: &str) -> Result<()> {
        let mut current_trade_symbol = None;

        loop {
            let ships = self.get_ships_at_waypoint(waypoint_symbol).await?;
            let (transport_ships, extraction_ships) = self.partition_ships_by_role(ships);

            let extraction_ships = self.filter_ships_with_cargo(extraction_ships);
            if extraction_ships.is_empty() {
                return Ok(());
            }

            let transport_ships = self.filter_ships_with_space(transport_ships);
            if transport_ships.is_empty() {
                return Ok(());
            }

            let trade_symbol = match current_trade_symbol {
                Some(symbol) => symbol,
                None => {
                    let symbol = self
                        .inventory_manager
                        .determine_most_abundant_cargo(&extraction_ships);
                    if symbol.is_none() {
                        return Ok(());
                    }
                    symbol
                }
            };
            current_trade_symbol = Some(trade_symbol.clone());

            let transfer_result = self
                .execute_transfer(
                    &transport_ships,
                    &extraction_ships,
                    trade_symbol.as_ref().unwrap(),
                )
                .await?;

            if !transfer_result {
                current_trade_symbol = None;
            }
        }
    }

    async fn get_ships_at_waypoint(&self, waypoint_symbol: &str) -> Result<Vec<ship::MyShip>> {
        Ok(self
            .context
            .ship_manager
            .get_all_clone()
            .await
            .into_iter()
            .filter(|f| f.1.nav.waypoint_symbol == waypoint_symbol)
            .filter(|f| ActionType::get_action(&f.1).is_some())
            .map(|f| f.1)
            .collect())
    }

    fn partition_ships_by_role(
        &self,
        ships: Vec<ship::MyShip>,
    ) -> (Vec<ship::MyShip>, Vec<ship::MyShip>) {
        ships.into_iter().partition(|f| {
            let action = ActionType::get_action(&f)
                .ok_or("Invalid ship role")
                .unwrap();
            action == ActionType::Extract
        })
    }

    fn filter_ships_with_cargo(&self, ships: Vec<ship::MyShip>) -> Vec<ship::MyShip> {
        ships.into_iter().filter(|f| f.cargo.units > 0).collect()
    }

    fn filter_ships_with_space(&self, ships: Vec<ship::MyShip>) -> Vec<ship::MyShip> {
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
        let transporter = match self
            .inventory_manager
            .find_best_transporter(transport_ships, trade_symbol)
        {
            Some(t) => t,
            None => return Ok(false),
        };

        let extractor = match self
            .inventory_manager
            .find_best_extractor(extraction_ships, trade_symbol)
        {
            Some(e) => e,
            None => return Ok(false),
        };

        let transfer_amount = std::cmp::min(extractor.amount, transporter.available_space);
        if transfer_amount > 0 {
            self.transfer_manager
                .process_transfer(
                    &extractor.ship_symbol,
                    &transporter.ship_symbol,
                    *trade_symbol,
                    transfer_amount,
                )
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn get_next_waypoint(&self, ship_clone: crate::ship::MyShip) -> Result<String> {
        let the_ships: std::collections::HashMap<String, ship::MyShip> =
            self.context.ship_manager.get_all_clone().await;
        let route = self
            .waypoint_manager
            .calculate_waypoint_urgencys(&the_ships);

        debug!("Routes: {:?}", route);
        let routes = route.into_iter().filter(|r| r.1 > 0).collect::<Vec<_>>();

        if routes.is_empty() {
            info!("No routes found for {}", ship_clone.symbol);
            return Err("No routes found".into());
        }

        let route = routes.last().unwrap();
        debug!("Route: {:?}", route);

        return Ok(route.0.clone());
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
