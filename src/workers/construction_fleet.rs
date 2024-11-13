use std::time::Duration;

use log::info;
use tokio::time::sleep;

pub struct ConstructionFleet {}

impl ConstructionFleet {
    pub fn new(context: super::types::ConductorContext) -> Self {
        ConstructionFleet {}
    }
}

impl super::types::Conductor for ConstructionFleet {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move {
            info!("Starting construction workers");
            sleep(Duration::from_secs(1)).await;

            info!("Construction workers done");

            Ok(())
        })
    }

    fn get_name(&self) -> String {
        "ConstructionFleet".to_string()
    }
}
