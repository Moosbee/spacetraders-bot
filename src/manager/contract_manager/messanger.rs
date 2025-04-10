use log::debug;

use crate::{
    error::{Error, Result},
    manager::{contract_manager::ContractShipmentMessage, fleet_manager::message::RequiredShips},
};

use super::{message::ContractManagerMessage, NextShipmentResp};

#[derive(Debug, Clone)]
pub struct ContractManagerMessanger {
    pub sender: tokio::sync::mpsc::Sender<ContractManagerMessage>,
}

impl ContractManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<ContractManagerMessage>) -> Self {
        Self { sender }
    }

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

    pub async fn complete_shipment(
        &self,
        shipment: database::ContractShipment,
        contract: space_traders_client::models::Contract,
    ) -> Result<()> {
        let message = ContractShipmentMessage::Finished { contract, shipment };

        debug!("Sending message: {:?}", message);

        self.sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    pub async fn get_ships(&self) -> Result<RequiredShips> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(ContractShipmentMessage::GetShips { callback: tx })
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))
            .unwrap();
        rx.await
            .map_err(|e| crate::error::Error::General(format!("Failed to receive message: {}", e)))
    }
}
