use std::collections::HashMap;

use log::{debug, info};
use space_traders_client::models::{self, trade_good};

use crate::{config::CONFIG, error::Result, manager::Manager, ship, types::WaypointCan};

use super::{
    mining_places::{AssignLevel, MiningPlaces},
    place_finder::{ActionType, PlaceFinder},
};

#[derive(Debug)]
pub enum AssignWaypointMessage {
    AssignWaypoint {
        // assigns a ship to a waypoint, ship might need to get there
        ship_clone: crate::ship::MyShip,
        is_syphon: bool,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
    NotifyWaypoint {
        // assigns a ship to a waypoint(level two), ship is now there
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
    UnassignWaypoint {
        // unassigns a ship from level two to level one
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
    #[allow(dead_code)]
    UnassignWaypointComplete {
        // unassigns a ship from level two to level one
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
}

#[derive(Debug)]
pub enum ExtractionNotification {
    GetNextWaypoint {
        // when a transporter is empty and wants to find a new waypoint, acording to it's urgency
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
    ExtractionComplete {
        // when a ship completed an extraction
        ship: String,
        waypoint: String,
    },
    TransportArrived {
        // when a transporter ship arrived
        ship: String,
        waypoint: String,
    },
}

#[derive(Debug)]
pub struct TransportTransferRequest {
    from_symbol: String,
    to_symbol: String,
    amount: i32,
    trade_symbol: models::TradeSymbol,
    extractor_contact: tokio::sync::mpsc::Sender<ExtractorTransferRequest>,
    callback: tokio::sync::oneshot::Sender<Result<()>>,
}

#[derive(Debug)]
pub struct ExtractorTransferRequest {
    from_symbol: String,
    to_symbol: String,
    amount: i32,
    trade_symbol: models::TradeSymbol,
    callback: tokio::sync::oneshot::Sender<Result<()>>,
}

struct TransferResult {
    ship_symbol: String,
    available_space: i32,
}

struct ShipInventory {
    ship_symbol: String,
    amount: i32,
}

#[derive(Debug)]
pub enum MiningMessage {
    AssignWaypoint(AssignWaypointMessage),
    ExtractionNotification(ExtractionNotification),
}

pub type MiningManagerMessage = MiningMessage;

#[derive(Debug)]
pub struct MiningManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: crate::workers::types::ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<MiningManagerMessage>,
    places: MiningPlaces,
    finder: PlaceFinder,
    extraction_contacts: HashMap<String, tokio::sync::mpsc::Sender<ExtractorTransferRequest>>,
    transportation_contacts: HashMap<String, tokio::sync::mpsc::Sender<TransportTransferRequest>>,
}

#[derive(Debug, Clone)]
pub struct MiningManagerMessanger {
    sender: tokio::sync::mpsc::Sender<MiningManagerMessage>,
}

impl MiningManagerMessanger {
    pub async fn get_waypoint(
        &self,
        ship: &crate::ship::MyShip,
        is_syphon: bool,
    ) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = MiningMessage::AssignWaypoint(AssignWaypointMessage::AssignWaypoint {
            ship_clone: ship.clone(),
            callback: sender,
            is_syphon,
        });
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        let erg = callback
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to get message: {}", e)))??;

        Ok(erg)
    }

    pub async fn notify_waypoint(&self, ship: &crate::ship::MyShip) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = MiningMessage::AssignWaypoint(AssignWaypointMessage::NotifyWaypoint {
            ship_clone: ship.clone(),
            callback: sender,
        });
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        let erg = callback
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to get message: {}", e)))??;

        Ok(erg)
    }

    pub async fn unassign_waypoint(&self, ship: &crate::ship::MyShip) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = MiningMessage::AssignWaypoint(AssignWaypointMessage::UnassignWaypoint {
            ship_clone: ship.clone(),
            callback: sender,
        });
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        let erg = callback
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to get message: {}", e)))??;

        Ok(erg)
    }

    pub async fn get_next_transport(&self, ship: &crate::ship::MyShip) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message =
            MiningMessage::ExtractionNotification(ExtractionNotification::GetNextWaypoint {
                ship_clone: ship.clone(),
                callback: sender,
            });
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        let erg = callback
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to get message: {}", e)))??;

        Ok(erg)
    }

    pub async fn extraction_complete(&self, ship: &str, waypoint: &str) -> Result<String> {
        let message =
            MiningMessage::ExtractionNotification(ExtractionNotification::ExtractionComplete {
                ship: ship.to_string(),
                waypoint: waypoint.to_string(),
            });
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        Ok("ok".to_string())
    }

    pub async fn transport_arrived(&self, ship: &str, waypoint: &str) -> Result<String> {
        let message =
            MiningMessage::ExtractionNotification(ExtractionNotification::TransportArrived {
                ship: ship.to_string(),
                waypoint: waypoint.to_string(),
            });
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        Ok("ok".to_string())
    }
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
            finder: PlaceFinder::new(context.clone()),
            context,
            receiver,
            places: MiningPlaces::new(CONFIG.mining.max_miners_per_waypoint),
            extraction_contacts: HashMap::new(),
            transportation_contacts: HashMap::new(),
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
                let erg = self.assign_waypoint(ship_clone, is_syphon).await;
                let _send = callback.send(erg);
            }
            // Handle notification that a ship has arrived at a waypoint
            AssignWaypointMessage::NotifyWaypoint {
                ship_clone,
                callback,
            } => {
                let erg = self.notify_waypoint(ship_clone).await;
                let _send = callback.send(erg);
            }
            // Handle unassigning a ship from a waypoint
            AssignWaypointMessage::UnassignWaypoint {
                ship_clone,
                callback,
            } => {
                let erg = self.unassign_waypoint(ship_clone).await;
                let _send = callback.send(erg);
            }
            // Handle completion of unassigning a ship
            AssignWaypointMessage::UnassignWaypointComplete {
                ship_clone,
                callback,
            } => {
                let erg = self.unassign_waypoint_complete(ship_clone).await;
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
            ExtractionNotification::ExtractionComplete { ship, waypoint } => {
                let _erg = self.process_possible_transfers(&waypoint).await?;
            }
            ExtractionNotification::TransportArrived { ship, waypoint } => {
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
                    let symbol = self.determine_most_abundant_cargo(&extraction_ships);
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

    fn determine_most_abundant_cargo(&self, ships: &[ship::MyShip]) -> Option<models::TradeSymbol> {
        let cargo_totals = ships
            .iter()
            .map(|f| f.cargo.inventory.clone())
            .reduce(|acc, inventory| {
                acc.into_iter()
                    .map(|(symbol, amount)| (symbol, amount + inventory.get(&symbol).unwrap_or(&0)))
                    .collect()
            })
            .unwrap_or_default();

        cargo_totals
            .into_iter()
            .max_by_key(|(_, amount)| *amount)
            .map(|(symbol, _)| symbol)
    }

    fn find_best_extractor(
        &self,
        ships: &[ship::MyShip],
        trade_symbol: &models::TradeSymbol,
    ) -> Option<ShipInventory> {
        ships
            .iter()
            .max_by_key(|ship| ship.cargo.get_amount(trade_symbol))
            .map(|ship| ShipInventory {
                ship_symbol: ship.symbol.clone(),
                amount: ship.cargo.get_amount(trade_symbol),
            })
    }

    fn find_best_transporter(
        &self,
        ships: &[ship::MyShip],
        trade_symbol: &models::TradeSymbol,
    ) -> Option<TransferResult> {
        // First try to find a ship that already has some of the cargo
        let transporter = ships
            .iter()
            .filter(|ship| ship.cargo.units < ship.cargo.capacity && ship.cargo.has(trade_symbol))
            .max_by_key(|ship| ship.cargo.get_amount(trade_symbol));

        // If none found, pick the one with most available space
        let ship = transporter.or_else(|| {
            ships
                .iter()
                .min_by_key(|ship| ship.cargo.capacity - ship.cargo.units)
        });

        ship.map(|ship| TransferResult {
            ship_symbol: ship.symbol.clone(),
            available_space: ship.cargo.capacity - ship.cargo.units,
        })
    }

    async fn execute_transfer(
        &mut self,
        transport_ships: &[ship::MyShip],
        extraction_ships: &[ship::MyShip],
        trade_symbol: &models::TradeSymbol,
    ) -> Result<bool> {
        let transporter = match self.find_best_transporter(transport_ships, trade_symbol) {
            Some(t) => t,
            None => return Ok(false),
        };

        let extractor = match self.find_best_extractor(extraction_ships, trade_symbol) {
            Some(e) => e,
            None => return Ok(false),
        };

        let transfer_amount = std::cmp::min(extractor.amount, transporter.available_space);
        if transfer_amount > 0 {
            self.transfer(
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

    async fn transfer(
        &mut self,
        from_extractor: &str,
        to_transporter: &str,
        symbol: models::TradeSymbol,
        amount: i32,
    ) -> Result<()> {
        let transporter = self
            .transportation_contacts
            .get(to_transporter)
            .ok_or("No transporter contact")?;

        let extractor = self
            .extraction_contacts
            .get(from_extractor)
            .ok_or("No extractor contact")?
            .clone();

        let (callback, rec) = tokio::sync::oneshot::channel();

        let request = TransportTransferRequest {
            from_symbol: from_extractor.to_string(),
            to_symbol: to_transporter.to_string(),
            amount,
            trade_symbol: symbol,
            extractor_contact: extractor,
            callback: callback,
        };

        match transporter.send(request).await {
            Ok(_) => {}
            Err(err) => {
                // transporter no longer receives requests, has left, remove it
                self.transportation_contacts.remove(to_transporter);
                return Err(format!("Transporter no longer receives requests: {}", err)
                    .as_str()
                    .into());
            }
        }

        let _erg = rec
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to get message: {}", e)))??;

        Ok(())
    }

    async fn assign_waypoint(
        &mut self,
        ship_clone: crate::ship::MyShip,
        is_syphon: bool,
    ) -> Result<String> {
        // let action: ActionType = ActionType::get_action(&ship_clone).ok_or("Invalid ship role")?;
        let action: ActionType = if is_syphon {
            ActionType::Siphon
        } else {
            ActionType::Extract
        };

        let assigned_waypoint = self.find_and_assign(ship_clone, action).await?;

        Ok(assigned_waypoint)
    }

    async fn find_and_assign(
        &mut self,
        ship_clone: crate::ship::MyShip,
        action: ActionType,
    ) -> Result<String> {
        let filter_fn = match action {
            ActionType::Extract => models::Waypoint::is_minable,
            ActionType::Siphon => models::Waypoint::is_sipherable,
        };

        let has = self.places.get_ship(&ship_clone.symbol);
        if let Some((waypoint_symbol, _assignment_level)) = has {
            let worked = self
                .places
                .try_assign_on_way(&ship_clone.symbol, &waypoint_symbol);

            if worked != 0 {
                return Ok(waypoint_symbol.to_string());
            }
        }

        let waypoints = self
            .finder
            .find(ship_clone.clone(), filter_fn, &self.places)
            .await;

        if waypoints.is_empty() {
            return Err("No waypoints found".into());
        }

        for waypoint in waypoints {
            let worked = self
                .places
                .try_assign_on_way(&ship_clone.symbol, &waypoint.waypoint.symbol);

            if worked == 0 {
                continue;
            }
            return Ok(waypoint.waypoint.symbol.clone());
        }

        Err("No suitable waypoints found".into())
    }

    async fn notify_waypoint(
        &mut self,
        ship_clone: crate::ship::MyShip,
    ) -> std::result::Result<String, crate::error::Error> {
        let waypoint_symbol = ship_clone.nav.waypoint_symbol.clone();

        let wp = self
            .places
            .try_assign_active(&ship_clone.symbol, &waypoint_symbol);

        if wp {
            return Ok(waypoint_symbol);
        } else {
            return Err("Could not activate craft".into());
        }
    }

    async fn unassign_waypoint(
        &mut self,
        ship_clone: crate::ship::MyShip,
    ) -> std::result::Result<String, crate::error::Error> {
        let waypoint_symbol = ship_clone.nav.waypoint_symbol.clone();

        let wp = self
            .places
            .try_assign_inactive(&ship_clone.symbol, &waypoint_symbol);

        if wp {
            return Ok(waypoint_symbol);
        } else {
            return Err("Could not deactivate craft".into());
        }
    }

    async fn unassign_waypoint_complete(
        &mut self,
        ship_clone: crate::ship::MyShip,
    ) -> std::result::Result<String, crate::error::Error> {
        let waypoint_symbol = ship_clone.nav.waypoint_symbol.clone();

        let wp = self
            .places
            .try_unassign(&ship_clone.symbol, &waypoint_symbol);

        if wp {
            return Ok(waypoint_symbol);
        } else {
            return Err("Could not deactivate craft".into());
        }
    }

    async fn calculate_waypoint_urgencys(&self) -> Vec<(String, u32)> {
        let the_ships: std::collections::HashMap<String, ship::MyShip> =
            self.context.ship_manager.get_all_clone().await;

        let mut erg = self
            .places
            .iter()
            .map(|wp| Self::calculate_waypoint_urgency(wp.1, the_ships.clone()))
            .collect::<Vec<_>>();

        erg.sort_by(|a, b| b.1.cmp(&a.1));

        erg
    }

    fn calculate_waypoint_urgency(
        wp: &super::mining_places::WaypointInfo,
        ships: std::collections::HashMap<String, ship::MyShip>,
    ) -> (String, u32) {
        let (units_sum, capacity_sum) = wp
            .ship_iter()
            .filter(|s| s.1 == &AssignLevel::Active)
            .map(|s| ships.get(s.0).unwrap())
            .filter(|sh| sh.nav.waypoint_symbol == wp.waypoint_symbol && !sh.nav.is_in_transit())
            .map(|sh| (sh.cargo.units, sh.cargo.capacity))
            .fold((0, 0), |(units_sum, capacity_sum), (units, capacity)| {
                (units_sum + units, capacity_sum + capacity)
            });

        let (_units_sum_on_way, _capacity_sum_on_way, earliest_arrival) = wp
            .ships_on_way
            .iter()
            .map(|s| ships.get(s).unwrap())
            .filter(|sh| {
                sh.nav.waypoint_symbol == wp.waypoint_symbol
                    || sh.nav.auto_pilot.as_ref().map(|a| &a.destination_symbol)
                        == Some(&wp.waypoint_symbol)
            })
            .map(|sh| {
                (
                    sh.cargo.units,
                    sh.cargo.capacity,
                    sh.nav.auto_pilot.as_ref().map(|a| a.arrival),
                )
            })
            .fold(
                (0, 0, chrono::DateTime::<chrono::Utc>::MAX_UTC),
                |(units_sum, capacity_sum, min_arrival), (units, capacity, arrival_time)| {
                    (
                        units_sum + units,
                        capacity_sum + capacity,
                        arrival_time
                            .unwrap_or(chrono::DateTime::<chrono::Utc>::MIN_UTC)
                            .min(min_arrival),
                    )
                },
            );

        let cargo_ratio = (units_sum as f32 / capacity_sum as f32) * 100.0;
        let cargo_ratio = if cargo_ratio.is_nan() {
            0.0
        } else {
            cargo_ratio
        };

        let since_last = wp.get_last_updated() - chrono::Utc::now();

        let to_next = (chrono::Utc::now() - earliest_arrival).max(chrono::Duration::seconds(0));

        let urgency = (since_last.num_seconds() + to_next.num_seconds()) * cargo_ratio as i64;

        (wp.waypoint_symbol.clone(), urgency as u32)
    }

    async fn get_next_waypoint(&self, ship_clone: crate::ship::MyShip) -> Result<String> {
        let route = self.calculate_waypoint_urgencys().await;

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
