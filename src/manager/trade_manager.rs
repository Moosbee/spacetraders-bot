use crate::error::Result;

use super::Manager;

type TradeManagerMessage = ();

#[derive(Debug)]
pub struct TradeManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: crate::workers::types::ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<TradeManagerMessage>,
}

#[derive(Debug, Clone)]
pub struct TradeManagerMessanger {
    sender: tokio::sync::mpsc::Sender<TradeManagerMessage>,
}

impl TradeManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<TradeManagerMessage>,
        TradeManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        (receiver, TradeManagerMessanger { sender })
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: crate::workers::types::ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<TradeManagerMessage>,
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

impl Manager for TradeManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_trade_worker().await })
    }

    fn get_name(&self) -> &str {
        "TradeManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
