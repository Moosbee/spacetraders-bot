use std::time::Duration;

use log::info;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{config::CONFIG, ship};

pub struct ConstructionFleet {
    context: super::types::ConductorContext,
    cancellation_token: CancellationToken,
}

impl ConstructionFleet {
    #[allow(dead_code)]
    pub fn new_box(_context: super::types::ConductorContext) -> Box<Self> {
        Box::new(ConstructionFleet {
            context: _context,
            cancellation_token: CancellationToken::new(),
        })
    }

    async fn run_construction_worker(&self) -> anyhow::Result<()> {
        info!("Starting construction workers");

        if !CONFIG.construction.active {
            info!("construction workers not active, exiting");

            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(
            CONFIG.construction.start_sleep_duration,
        ))
        .await;

        sleep(Duration::from_secs(1)).await;

        let _ships = self.get_construction_ships();

        info!("Construction workers done");

        Ok(())
    }

    fn get_construction_ships(&self) -> Vec<String> {
        self.context
            .ship_roles
            .iter()
            .filter(|(_, role)| **role == ship::Role::Construction)
            .map(|(symbol, _)| symbol.clone())
            .collect()
    }
}

impl super::types::Conductor for ConstructionFleet {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_construction_worker().await })
    }

    fn get_name(&self) -> String {
        "ConstructionFleet".to_string()
    }

    fn get_cancel_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }
}
