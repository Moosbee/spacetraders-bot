use std::sync::Arc;

use tracing::debug;

use super::{
    mining_messages::{
        AssignWaypointMessage, ExtractionNotification, MiningManagerMessage, MiningMessage,
    },
    transfer_manager::{ExtractorTransferRequest, TransferManager, TransportTransferRequest},
    MiningManager,
};

use crate::{error::Result, manager::fleet_manager::message::RequiredShips};

#[derive(Debug, Clone)]
pub struct MiningManagerMessanger {
    pub(crate) sender: tokio::sync::mpsc::Sender<MiningManagerMessage>,
    transfer_manager: Arc<TransferManager>,
}

impl MiningManagerMessanger {
    pub fn new(
        sender: tokio::sync::mpsc::Sender<MiningManagerMessage>,
        transfer_manager: Arc<TransferManager>,
    ) -> Self {
        MiningManagerMessanger {
            sender,
            transfer_manager,
        }
    }

    #[tracing::instrument(skip(self, ship), name = "MiningManagerMessanger::get_waypoint", fields(ship = %ship.symbol))]
    pub async fn get_waypoint(&self, ship: &ship::MyShip, is_syphon: bool) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = MiningMessage::AssignWaypoint(AssignWaypointMessage::AssignWaypoint {
            ship_clone: ship.clone(),
            callback: sender,
            is_syphon,
        });
        debug!("Sending AssignWaypoint message for ship: {}", ship.symbol);
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send AssignWaypoint message: {}", e))
        })?;

        let erg = callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to get AssignWaypoint message: {}", e))
        })??;

        debug!("Received waypoint: {}", erg);
        Ok(erg)
    }

    #[tracing::instrument(skip(self, ship), name = "MiningManagerMessanger::notify_waypoint", fields(ship = %ship.symbol))]
    pub async fn notify_waypoint(&self, ship: &ship::MyShip, is_syphon: bool) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = MiningMessage::AssignWaypoint(AssignWaypointMessage::NotifyWaypoint {
            ship_clone: ship.clone(),
            callback: sender,
            is_syphon,
        });
        debug!("Sending NotifyWaypoint message for ship: {}", ship.symbol);
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send NotifyWaypoint message: {}", e))
        })?;

        let erg = callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to get NotifyWaypoint message: {}", e))
        })??;

        debug!("Received notification response for ship: {}", ship.symbol);
        Ok(erg)
    }

    #[tracing::instrument(skip(self, ship), name = "MiningManagerMessanger::unassign_waypoint", fields(ship = %ship.symbol))]
    pub async fn unassign_waypoint(&self, ship: &ship::MyShip) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = MiningMessage::AssignWaypoint(AssignWaypointMessage::UnassignWaypoint {
            ship_clone: ship.clone(),
            callback: sender,
        });
        debug!("Sending UnassignWaypoint message for ship: {}", ship.symbol);
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send UnassignWaypoint message: {}", e))
        })?;

        let erg = callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to get UnassignWaypoint message: {}", e))
        })??;

        debug!("Received unassignment response for ship: {}", ship.symbol);
        Ok(erg)
    }

    #[tracing::instrument(skip(self, ship), name = "MiningManagerMessanger::get_next_transport", fields(ship = %ship.symbol))]
    pub async fn get_next_transport(&self, ship: &ship::MyShip) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message =
            MiningMessage::ExtractionNotification(ExtractionNotification::GetNextWaypoint {
                ship_clone: ship.clone(),
                callback: sender,
            });
        debug!("Sending GetNextWaypoint message for ship: {}", ship.symbol);
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send GetNextWaypoint message: {}", e))
        })?;

        let erg = callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to get GetNextWaypoint message: {}", e))
        })??;

        debug!("Received next transport waypoint: {}", erg);
        Ok(erg)
    }

    #[tracing::instrument(skip(self, ship, waypoint), name = "MiningManagerMessanger::extraction_complete", fields(ship = %ship, waypoint = %waypoint))]
    pub async fn extraction_complete(&self, ship: &str, waypoint: &str) -> Result<String> {
        let message =
            MiningMessage::ExtractionNotification(ExtractionNotification::ExtractionComplete {
                ship: ship.to_string(),
                waypoint: waypoint.to_string(),
            });
        debug!(
            "Sending ExtractionComplete message for ship: {}, waypoint: {}",
            ship, waypoint
        );
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!(
                "Failed to send ExtractionComplete message: {}",
                e
            ))
        })?;

        debug!(
            "Extraction complete for ship: {}, waypoint: {}",
            ship, waypoint
        );
        Ok("ok".to_string())
    }

    #[tracing::instrument(skip(self, ship, waypoint), name = "MiningManagerMessanger::transport_arrived", fields(ship = %ship, waypoint = %waypoint))]
    pub async fn transport_arrived(&self, ship: &str, waypoint: &str) -> Result<String> {
        let message =
            MiningMessage::ExtractionNotification(ExtractionNotification::TransportArrived {
                ship: ship.to_string(),
                waypoint: waypoint.to_string(),
            });
        debug!(
            "Sending TransportArrived message for ship: {}, waypoint: {}",
            ship, waypoint
        );
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send TransportArrived message: {}", e))
        })?;

        debug!(
            "Transport arrived for ship: {}, waypoint: {}",
            ship, waypoint
        );
        Ok("ok".to_string())
    }

    #[tracing::instrument(
        skip(self),
        name = "MiningManagerMessanger::extractor_contact",
        fields()
    )]
    pub async fn extractor_contact(
        &self,
        symbol: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<ExtractorTransferRequest>> {
        let (sender, receiver) = tokio::sync::mpsc::channel(32);

        self.transfer_manager.add_extractor_contact(symbol, sender);

        debug!("Extractor contact established for symbol: {}", symbol);
        Ok(receiver)
    }

    #[tracing::instrument(
        skip(self),
        name = "MiningManagerMessanger::transport_contact",
        fields()
    )]
    pub async fn transport_contact(
        &self,
        symbol: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<TransportTransferRequest>> {
        let (sender, receiver) = tokio::sync::mpsc::channel(32);

        self.transfer_manager
            .add_transportation_contact(symbol, sender);

        debug!("Transport contact established for symbol: {}", symbol);
        Ok(receiver)
    }

    #[tracing::instrument(skip(self), name = "MiningManagerMessanger::get_assignments")]
    pub async fn get_assignments(
        &self,
    ) -> Result<Vec<(String, super::mining_places::WaypointInfo)>> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = MiningMessage::GetPlaces { callback: sender };
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send GetAssignments message: {}", e))
        })?;

        callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to get GetAssignments message: {}", e))
        })?
    }

    pub async fn get_ships(
        &self,
        context: &crate::utils::ConductorContext,
    ) -> Result<RequiredShips> {
        MiningManager::get_required_ships(context).await
    }
}
