use crate::error::Result;

use super::Manager;

type ContractManagerMessage = ();

#[derive(Debug)]
pub struct ContractManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: crate::workers::types::ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ContractManagerMessage>,
}

#[derive(Debug, Clone)]
pub struct ContractManagerMessanger {
    sender: tokio::sync::mpsc::Sender<ContractManagerMessage>,
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

    async fn run_trade_worker(&self) -> Result<()> {
        todo!()
    }
}

impl Manager for ContractManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_trade_worker().await })
    }

    fn get_name(&self) -> &str {
        "ContractManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
