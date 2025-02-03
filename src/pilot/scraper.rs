use crate::error::Result;

pub struct ScraperPilot {}

impl ScraperPilot {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn execute_pilot_circle(&self, pilot: &super::Pilot) -> Result<()> {
        pilot.cancellation_token.cancelled().await;
        Ok(())
    }
}
