use log::debug;
use space_traders_client::models;

use crate::{config::CONFIG, error::Result, manager::Manager, types::WaypointCan};

use super::{
    mining_places::MiningPlaces,
    place_finder::{ActionType, PlaceFinder},
};

#[derive(Debug)]
pub enum AssignWaypointMessage {
    AssignWaypoint {
        // assigns a ship to a waypoint, ship might need to get there
        ship_clone: crate::ship::MyShip,
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
    UnassignWaypointComplete {
        // unassigns a ship from level two to level one
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
}

#[derive(Debug)]
pub enum ExtractionNotification {
    GetNextWaypoint {
        // when a transporter is empty and wants to find a new waypoint
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
    ExtractionComplete {
        // when a ship completed an extraction
        ship: String,
    },
    TransportArrived {
        // when a transporter ship arrived
        ship: String,
    },
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
}

#[derive(Debug, Clone)]
pub struct MiningManagerMessanger {
    sender: tokio::sync::mpsc::Sender<MiningManagerMessage>,
}

impl MiningManagerMessanger {
    pub async fn get_waypoint(&self, ship: &crate::ship::MyShip) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = MiningMessage::AssignWaypoint(AssignWaypointMessage::AssignWaypoint {
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
            AssignWaypointMessage::AssignWaypoint {
                ship_clone,
                callback,
            } => {
                let erg = self.assign_waypoint(ship_clone).await;

                let _send = callback.send(erg);
            }
            AssignWaypointMessage::NotifyWaypoint {
                ship_clone,
                callback,
            } => {
                let erg = self.notify_waypoint(ship_clone).await;

                let _send = callback.send(erg);
            }
            AssignWaypointMessage::UnassignWaypoint {
                ship_clone,
                callback,
            } => {
                let erg = self.unassign_waypoint(ship_clone).await;

                let _send = callback.send(erg);
            }
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

    async fn handle_extraction_notification(&self, message: ExtractionNotification) -> Result<()> {
        match message {
            ExtractionNotification::GetNextWaypoint {
                ship_clone,
                callback,
            } => todo!(),
            ExtractionNotification::ExtractionComplete { ship } => todo!(),
            ExtractionNotification::TransportArrived { ship } => todo!(),
        }
        Ok(())
    }

    async fn assign_waypoint(&mut self, ship_clone: crate::ship::MyShip) -> Result<String> {
        let action: ActionType = ActionType::get_action(&ship_clone).ok_or("Invalid ship role")?;

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
        if let Some((waypoint_symbol, assignment_level)) = has {
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
