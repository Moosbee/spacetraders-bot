use std::time::Duration;

use log::info;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{config::CONFIG, ship};

pub struct MiningFleet {
    context: super::types::ConductorContext,
    cancellation_token: CancellationToken,
}

impl MiningFleet {
    #[allow(dead_code)]
    pub fn new_box(_context: super::types::ConductorContext) -> Box<Self> {
        Box::new(MiningFleet {
            context: _context,
            cancellation_token: CancellationToken::new(),
        })
    }

    async fn run_mining_worker(&self) -> anyhow::Result<()> {
        info!("Starting mining workers");

        if !CONFIG.mining.active {
            info!("mining workers not active, exiting");
            return Ok(());
        }

        tokio::select! {
        _ = self.cancellation_token.cancelled() => {
          info!("Agent scrapping cancelled");
          0},
        _ =  sleep(Duration::from_millis(CONFIG.mining.start_sleep_duration)) => {1}
        };

        let _ships = self.get_mining_ships();

        info!("mining workers done");

        Ok(())
    }

    fn get_mining_ships(&self) -> Vec<String> {
        self.context
            .ship_roles
            .iter()
            .filter(|(_, role)| **role == ship::Role::Mining)
            .map(|(symbol, _)| symbol.clone())
            .collect()
    }
}

impl super::types::Conductor for MiningFleet {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_mining_worker().await })
    }

    fn get_name(&self) -> String {
        "MiningFleet".to_string()
    }
    fn get_cancel_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }
}
