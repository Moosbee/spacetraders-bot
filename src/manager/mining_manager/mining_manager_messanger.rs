use super::{
    mining_messages::{
        AssignWaypointMessage, ExtractionNotification, MiningManagerMessage, MiningMessage,
    },
    transfer_manager::{ExtractorTransferRequest, TransportTransferRequest},
};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct MiningManagerMessanger {
    pub(crate) sender: tokio::sync::mpsc::Sender<MiningManagerMessage>,
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

    pub async fn extractor_contact(
        &self,
        symbol: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<ExtractorTransferRequest>> {
        let (sender, receiver) = tokio::sync::mpsc::channel(32);

        let message =
            MiningMessage::ExtractionNotification(ExtractionNotification::ExtractorContact {
                symbol: symbol.to_string(),
                sender,
            });

        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        Ok(receiver)
    }
    pub async fn transport_contact(
        &self,
        symbol: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<TransportTransferRequest>> {
        let (sender, receiver) = tokio::sync::mpsc::channel(32);

        let message =
            MiningMessage::ExtractionNotification(ExtractionNotification::TransportationContact {
                symbol: symbol.to_string(),
                sender,
            });

        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        Ok(receiver)
    }
}
