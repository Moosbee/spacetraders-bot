use std::sync::{atomic::AtomicBool, Arc};
use tracing::debug;

use crate::{
    error::{Error, Result},
    manager::contract_manager::ContractShipmentMessage,
};

use super::{message::ContractManagerMessage, NextShipmentResp};

#[derive(Debug, Clone)]
pub struct ContractManagerMessanger {
    pub sender: tokio::sync::mpsc::Sender<ContractManagerMessage>,
    busy: Arc<AtomicBool>,
}

impl ContractManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<ContractManagerMessage>) -> Self {
        Self {
            sender,
            busy: Arc::new(AtomicBool::new(false)),
        }
    }

    #[tracing::instrument(skip(self, ship), name = "ContractManagerMessanger::request_next_shipment", fields(ship = %ship.symbol))]
    pub async fn request_next_shipment(&self, ship: &ship::MyShip) -> Result<NextShipmentResp> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let message = ContractShipmentMessage::RequestNext {
            ship_clone: ship.clone(),
            callback: sender,
            can_start_new_contract: true,
        };
        self.sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver.await.map_err(|e| {
            Error::General(format!("Failed to get contract request message: {}", e))
        })?;

        debug!("Got response: {:?}", resp);

        resp
    }

    #[tracing::instrument(skip(self), name = "ContractManagerMessanger::fail_shipment", fields(shipment_id = %shipment.id))]
    pub async fn fail_shipment(
        &self,
        shipment: database::ContractShipment,
        error: Error,
    ) -> Result<Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let message = ContractShipmentMessage::Failed {
            shipment,
            error,
            callback: sender,
        };
        self.sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get contract fail message: {}", e)))?;

        debug!("Got response: {:?}", resp);

        resp
    }

    #[tracing::instrument(skip(self, contract), name = "ContractManagerMessanger::complete_shipment", fields(shipment_id = %shipment.id))]
    pub async fn complete_shipment(
        &self,
        shipment: database::ContractShipment,
        contract: space_traders_client::models::Contract,
    ) -> Result<()> {
        let message = ContractShipmentMessage::Finished { contract, shipment };

        self.sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self), name = "ContractManagerMessanger::get_running_shipments")]
    pub async fn get_running_shipments(&self) -> Result<Vec<database::ContractShipment>> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let message = ContractShipmentMessage::GetRunning { callback: sender };
        self.sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get contract fail message: {}", e)))?;

        debug!("Got response: {:?}", resp);

        resp
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
