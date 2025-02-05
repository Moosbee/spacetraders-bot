use std::sync::{atomic::AtomicI32, Arc};

use crate::{error::Result, ship, workers::types::ConductorContext};

pub struct TransportPilot {
    count: Arc<AtomicI32>,
    context: ConductorContext,
}

impl TransportPilot {
    pub fn new(context: ConductorContext) -> Self {
        Self {
            count: Arc::new(AtomicI32::new(0)),
            context,
        }
    }

    pub async fn execute_transport_circle(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        pilot.cancellation_token.cancelled().await;
        Ok(())
    }
}
