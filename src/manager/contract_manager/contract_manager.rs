use crate::{error::Result, manager::Manager};

use super::ContractShipment;

pub enum ContractMessage {
    RequestNextShipment {
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<ContractShipment>>,
    },
    FailedShipment {
        shipment: ContractShipment,
        error: crate::error::Error,
        callback: tokio::sync::oneshot::Sender<Result<crate::error::Error>>,
    },
    FinishedShipment {
        contract: space_traders_client::models::Contract,
        shipment: ContractShipment,
    },
}

type ContractManagerMessage = ContractMessage;

#[derive(Debug)]
pub struct ContractManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: crate::workers::types::ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ContractManagerMessage>,
}

#[derive(Debug, Clone)]
pub struct ContractManagerMessanger {
    pub sender: tokio::sync::mpsc::Sender<ContractManagerMessage>,
}

impl ContractManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<ContractManagerMessage>,
        ContractManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, ContractManagerMessanger { sender })
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: crate::workers::types::ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<ContractManagerMessage>,
    ) -> Self {
        Self {
            cancel_token,
            context,
            receiver,
        }
    }

    async fn run_contract_worker(&mut self) -> Result<()> {
        while !self.cancel_token.is_cancelled() {
            let message = self.receiver.recv().await;

            match message {
                Some(message) => {
                    self.handle_contract_message(message).await?;
                }
                None => break,
            }
        }

        Ok(())
    }

    async fn handle_contract_message(&self, message: ContractManagerMessage) -> Result<()> {
        todo!()
    }
}

impl Manager for ContractManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_contract_worker().await })
    }

    fn get_name(&self) -> &str {
        "ContractManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
