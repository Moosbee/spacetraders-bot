use space_traders_client::models;
use std::sync::{atomic::AtomicBool, Arc};

use super::message::{self, ConstructionManagerMessage};

#[derive(Debug, Clone)]
pub struct ConstructionManagerMessanger {
    sender: tokio::sync::mpsc::Sender<ConstructionManagerMessage>,
    busy: Arc<AtomicBool>,
}

impl ConstructionManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<ConstructionManagerMessage>) -> Self {
        Self {
            sender,
            busy: Arc::new(AtomicBool::new(false)),
        }
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
