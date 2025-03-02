use crate::{error::Result, types::ConductorContext};

use super::Manager;

type ConstructionManagerMessage = ();

#[derive(Debug)]
pub struct ConstructionManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ConstructionManagerMessage>,
}

#[derive(Debug, Clone)]
pub struct ConstructionManagerMessanger {
    sender: tokio::sync::mpsc::Sender<ConstructionManagerMessage>,
}

impl ConstructionManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<ConstructionManagerMessage>,
        ConstructionManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, ConstructionManagerMessanger { sender })
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<ConstructionManagerMessage>,
    ) -> Self {
        Self {
            cancel_token,
            context,
            receiver,
        }
    }

    async fn run_construction_worker(&self) -> Result<()> {
        todo!()
    }
}

impl Manager for ConstructionManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_construction_worker().await })
    }

    fn get_name(&self) -> &str {
        "ConstructionManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
