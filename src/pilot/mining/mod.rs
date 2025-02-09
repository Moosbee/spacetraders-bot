mod extraction;
mod transport;

use log::debug;

use crate::{
    error::{Error, Result},
    ship::{self, Role},
    workers::{mining::m_types::MiningShipAssignment, types::ConductorContext},
};

pub struct MiningPilot {
    extraction: extraction::ExtractionPilot,
    transport: transport::TransportPilot,
    context: ConductorContext,
    ship_symbol: String,
}

impl MiningPilot {
    pub fn new(context: ConductorContext, ship_symbol: String) -> Self {
        Self {
            ship_symbol,
            extraction: extraction::ExtractionPilot::new(context.clone()),
            transport: transport::TransportPilot::new(context.clone()),
            context,
        }
    }

    pub async fn execute_pilot_circle(&self, pilot: &super::Pilot) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        self.assign_ships(ship).await;

        if let Role::Mining(assignment) = ship.role {
            match assignment {
                MiningShipAssignment::Extractor => {
                    self.run_extractor_ship_worker(ship, pilot).await?
                }
                MiningShipAssignment::Transporter => {
                    self.run_transporter_ship_worker(ship, pilot).await?
                }
                MiningShipAssignment::Siphoner => {
                    self.run_siphoned_ship_worker(ship, pilot).await?
                }
                MiningShipAssignment::Surveyor => {
                    self.run_surveyor_ship_worker(ship, pilot).await?
                }
                MiningShipAssignment::Idle => {}
                MiningShipAssignment::Useless => {}
            }
        }

        Ok(())
    }

    async fn assign_ships(&self, ship: &mut ship::MyShip) {
        let ship_capabilities = self.analyze_ship_capabilities(ship);

        ship.role = match ship_capabilities {
            ShipCapabilities {
                can_extract: true,
                can_cargo: true,
                ..
            } => Role::Mining(MiningShipAssignment::Extractor),

            ShipCapabilities {
                can_extract: false,
                can_siphon: true,
                can_cargo: true,
                ..
            } => Role::Mining(MiningShipAssignment::Siphoner),

            ShipCapabilities {
                can_survey: true, ..
            } => Role::Mining(MiningShipAssignment::Surveyor),

            ShipCapabilities {
                can_cargo: true, ..
            } => Role::Mining(MiningShipAssignment::Transporter),

            _ => Role::Mining(MiningShipAssignment::Useless),
        };

        debug!("Assigning role {:?} to ship {}", ship.role, ship.symbol);

        ship.notify().await;
    }
    fn analyze_ship_capabilities(&self, ship: &ship::MyShip) -> ShipCapabilities {
        ShipCapabilities {
            can_extract: ship.mounts.can_extract(),
            can_siphon: ship.mounts.can_siphon(),
            can_survey: ship.mounts.can_survey(),
            can_cargo: ship.cargo.capacity > 0,
        }
    }

    async fn run_extractor_ship_worker(
        &self,
        ship: &mut ship::MyShip,
        pilot: &super::Pilot,
    ) -> Result<()> {
        self.extraction
            .execute_extraction_circle(ship, pilot, false)
            .await
    }

    async fn run_transporter_ship_worker(
        &self,
        ship: &mut ship::MyShip,
        pilot: &super::Pilot,
    ) -> Result<()> {
        self.transport.execute_transport_circle(ship, pilot).await
    }

    async fn run_siphoned_ship_worker(
        &self,
        ship: &mut ship::MyShip,
        pilot: &super::Pilot,
    ) -> Result<()> {
        self.extraction
            .execute_extraction_circle(ship, pilot, true)
            .await
    }

    async fn run_surveyor_ship_worker(
        &self,
        ship: &mut ship::MyShip,
        pilot: &super::Pilot,
    ) -> Result<()> {
        pilot.cancellation_token.cancelled().await;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ShipCapabilities {
    can_extract: bool,
    can_siphon: bool,
    can_survey: bool,
    can_cargo: bool,
}
