use std::sync::{atomic::AtomicBool, Arc};

use tracing::debug;

use super::{
    mining_messages::{
        AssignWaypointMessage, ExtractionNotification, MiningManagerMessage, MiningMessage,
    },
    transfer_manager::{ExtractorTransferRequest, TransferManager, TransportTransferRequest},
};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct MiningManagerMessanger {
    pub(crate) sender: tokio::sync::mpsc::Sender<MiningManagerMessage>,
    transfer_manager: Arc<TransferManager>,
    busy: Arc<AtomicBool>,
}

impl MiningManagerMessanger {
    pub fn new(
        sender: tokio::sync::mpsc::Sender<MiningManagerMessage>,
        transfer_manager: Arc<TransferManager>,
    ) -> Self {
        MiningManagerMessanger {
            sender,
            transfer_manager,
            busy: Arc::new(AtomicBool::new(false)),
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
        tracing::debug!(ship_symbol = %ship.symbol, "Sending AssignWaypoint message for ship");
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send AssignWaypoint message: {}", e))
        })?;

        let erg = callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to get AssignWaypoint message: {}", e))
        })??;

        tracing::debug!(waypoint = %erg, "Received waypoint");
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
        tracing::debug!(ship_symbol = %ship.symbol, "Sending NotifyWaypoint message for ship");
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send NotifyWaypoint message: {}", e))
        })?;

        let erg = callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to get NotifyWaypoint message: {}", e))
        })??;

        tracing::debug!(ship_symbol = %ship.symbol, "Received notification response for ship");
        Ok(erg)
    }

    #[tracing::instrument(skip(self, ship), name = "MiningManagerMessanger::unassign_waypoint", fields(ship = %ship.symbol))]
    pub async fn unassign_waypoint(&self, ship: &ship::MyShip) -> Result<String> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = MiningMessage::AssignWaypoint(AssignWaypointMessage::UnassignWaypoint {
            ship_clone: ship.clone(),
            callback: sender,
        });
        tracing::debug!(ship_symbol = %ship.symbol, "Sending UnassignWaypoint message for ship");
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send UnassignWaypoint message: {}", e))
        })?;

        let erg = callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to get UnassignWaypoint message: {}", e))
        })??;

        tracing::debug!(ship_symbol = %ship.symbol, "Received unassignment response for ship");
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
        tracing::debug!(ship_symbol = %ship.symbol, "Sending GetNextWaypoint message for ship");
        self.sender.send(message).await.map_err(|e| {
            crate::error::Error::General(format!("Failed to send GetNextWaypoint message: {}", e))
        })?;

        let erg = callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to get GetNextWaypoint message: {}", e))
        })??;

        tracing::debug!(waypoint = %erg, "Received next transport waypoint");
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

        tracing::debug!(symbol = %symbol, "Extractor contact established");
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

        tracing::debug!(symbol = %symbol, "Transport contact established");
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

    pub fn is_busy(&self) -> bool {
        self.busy.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn set_busy(&self, busy: bool) -> bool {
        self.busy.swap(busy, std::sync::atomic::Ordering::Relaxed)
    }
    pub fn get_channel_state(&self) -> crate::utils::ChannelInfo {
        let state = if self.sender.is_closed() {
            crate::utils::ChannelState::Closed
        } else {
            crate::utils::ChannelState::Open
        };

        let max_capacity = self.sender.max_capacity();
        let free_capacity = self.sender.capacity();
        let used_capacity = max_capacity - free_capacity;

        crate::utils::ChannelInfo {
            state,
            total_capacity: max_capacity,
            used_capacity,
            free_capacity,
        }
    }
}
