use space_traders_client::models;

use crate::manager::fleet_manager::message::RequiredShips;

use super::{
    message::{self, ConstructionManagerMessage},
    ConstructionManager,
};

#[derive(Debug, Clone)]
pub struct ConstructionManagerMessanger {
    sender: tokio::sync::mpsc::Sender<ConstructionManagerMessage>,
}

impl ConstructionManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<ConstructionManagerMessage>) -> Self {
        Self { sender }
    }

    #[tracing::instrument(skip(self, ship_clone), name = "ConstructionManagerMessanger::next_shipment", fields(ship = %ship_clone.symbol))]
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

    #[tracing::instrument(skip(self), name = "ConstructionManagerMessanger::fail_shipment", fields(shipment_id = %shipment.id))]
    pub async fn fail_shipment(
        &self,
        shipment: database::ConstructionShipment,
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

    #[tracing::instrument(skip(self), name = "ConstructionManagerMessanger::complete_shipment", fields(shipment_id = %shipment.id))]
    pub async fn complete_shipment(
        &self,
        shipment: database::ConstructionShipment,
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

    pub async fn get_running_shipments(
        &self,
    ) -> Result<Vec<database::ConstructionShipment>, crate::error::Error> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = message::ConstructionManagerMessage::GetRunning { callback: sender };
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        let resp = callback.await.map_err(|e| {
            crate::error::Error::General(format!(
                "Failed to get construction get_running_shipments message: {}",
                e
            ))
        })??;

        Ok(resp)
    }

    pub async fn get_ships(
        &self,
        context: &crate::utils::ConductorContext,
    ) -> Result<RequiredShips, crate::error::Error> {
        ConstructionManager::get_required_ships(context).await
    }
}
