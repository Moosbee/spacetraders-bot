use crate::error::Result;


pub struct ContractPilot {}

impl ContractPilot {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn execute_pilot_circle(&self, pilot: &super::Pilot) -> Result<()> {
        Ok(())
    }
}
