use crate::error::Result;

use super::Manager;

type ScrappingManagerMessage = ();

#[derive(Debug)]
pub struct ScrappingManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: crate::workers::types::ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<ScrappingManagerMessage>,
}

#[derive(Debug, Clone)]
pub struct ScrappingManagerMessanger {
    sender: tokio::sync::mpsc::Sender<ScrappingManagerMessage>,
}

impl ScrappingManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<ScrappingManagerMessage>,
        ScrappingManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, ScrappingManagerMessanger { sender })
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: crate::workers::types::ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<ScrappingManagerMessage>,
    ) -> Self {
        Self {
            cancel_token,
            context,
            receiver,
        }
    }

    async fn run_scrapping_worker(&self) -> Result<()> {
        self.cancel_token.cancelled().await;

        Ok(())
    }
}

impl Manager for ScrappingManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_scrapping_worker().await })
    }

    fn get_name(&self) -> &str {
        "ScrappingManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
