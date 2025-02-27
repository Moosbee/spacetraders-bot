use std::time::Duration;

use log::{error, info};

use crate::{config::CONFIG, error::Result, manager::Manager};

use super::{agent_scrapper, market_scrapper};

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
        tokio::time::sleep(Duration::from_millis(CONFIG.market.start_sleep_duration)).await;
        let agent_scrapper = agent_scrapper::AgentScrapper::new(
            self.cancel_token.child_token(),
            self.context.clone(),
        );
        let market_scrapper = market_scrapper::MarketScrapper::new(
            self.cancel_token.child_token(),
            self.context.clone(),
        );

        let (_erg1, _erg2) = tokio::join!(
            agent_scrapper.run_scrapping_worker(),
            market_scrapper.run_scrapping_worker()
        );

        if let Err(err) = _erg1 {
            error!("Agent scrapper error: {}", err);
        }

        if let Err(err) = _erg2 {
            error!("Market scrapper error: {}", err);
        }

        info!("ScrappingManager done");

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
