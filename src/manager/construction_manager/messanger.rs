use space_traders_client::models;

use crate::{ship, sql};

use super::message::{self, ConstructionManagerMessage};

#[derive(Debug, Clone)]
pub struct ConstructionManagerMessanger {
    sender: tokio::sync::mpsc::Sender<ConstructionManagerMessage>,
}

impl ConstructionManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<ConstructionManagerMessage>) -> Self {
        Self { sender }
    }

    pub async fn next_shipment(
        &self,
        ship_clone: ship::MyShip,
    ) -> Result<super::message::NextShipmentResp, crate::error::Error> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = message::ConstructionManagerMessage::RequestNextShipment {
            ship_clone,
            callback: sender,
        };

        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        let resp = callback.await.map_err(|e| {
            crate::error::Error::General(format!(
                "Failed to get construction next_shipment message: {}",
                e
            ))
        })??;

        Ok(resp)
    }

    pub async fn fail_shipment(
        &self,
        shipment: sql::ConstructionShipment,
        error: crate::error::Error,
    ) -> Result<crate::error::Error, crate::error::Error> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = message::ConstructionManagerMessage::FailedShipment {
            shipment,
            error,
            callback: sender,
        };
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        let resp = callback.await.map_err(|e| {
            crate::error::Error::General(format!(
                "Failed to get construction fail_shipment message: {}",
                e
            ))
        })?;

        Ok(resp)
    }

    pub async fn complete_shipment(
        &self,
        shipment: sql::ConstructionShipment,
        construction: models::Construction,
    ) -> Result<(), crate::error::Error> {
        let message = message::ConstructionManagerMessage::FinishedShipment {
            shipment,
            construction,
        };
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        Ok(())
    }
}
