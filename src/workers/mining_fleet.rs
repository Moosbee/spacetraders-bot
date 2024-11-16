use std::time::Duration;

use log::info;
use tokio::time::sleep;

pub struct MiningFleet {}

impl MiningFleet {
    #[allow(dead_code)]
    pub fn new_box(_context: super::types::ConductorContext) -> Box<Self> {
        Box::new(MiningFleet {})
    }
}

impl super::types::Conductor for MiningFleet {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move {
            info!("Starting mining workers");
            sleep(Duration::from_secs(1)).await;

            info!("Mining workers done");

            Ok(())
        })
    }

    fn get_name(&self) -> String {
        "MiningFleet".to_string()
    }
}
