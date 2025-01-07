use crate::error::Result;

pub struct TradingPilot {}

impl TradingPilot {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn execute_pilot_circle(&self, pilot: &crate::pilot::Pilot) -> Result<()> {
        Ok(())
    }
}
