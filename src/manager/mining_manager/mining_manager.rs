use crate::{error::Result, manager::Manager};

pub type MiningManagerMessage = ();

#[derive(Debug)]
pub struct MiningManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: crate::workers::types::ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<MiningManagerMessage>,
}

#[derive(Debug, Clone)]
pub struct MiningManagerMessanger {
    sender: tokio::sync::mpsc::Sender<MiningManagerMessage>,
}

impl MiningManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<MiningManagerMessage>,
        MiningManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, MiningManagerMessanger { sender })
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: crate::workers::types::ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<MiningManagerMessage>,
    ) -> Self {
        Self {
            cancel_token,
            context,
            receiver,
        }
    }

    async fn run_mining_worker(&self) -> Result<()> {
        Ok(())
    }
}

impl Manager for MiningManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_mining_worker().await })
    }

    fn get_name(&self) -> &str {
        "MiningManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
