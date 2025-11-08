mod extraction;
mod surveyor;
mod transport;

use ship::status::MiningShipAssignment;
use tracing::debug;
use tracing::instrument;

use crate::{
    error::{Error, Result},
    utils::ConductorContext,
};

pub struct MiningPilot {
    extraction: extraction::ExtractionPilot,
    transport: transport::TransportPilot,
    survey: surveyor::SurveyPilot,
    context: ConductorContext,
    ship_symbol: String,
}

impl MiningPilot {
    pub fn new(context: ConductorContext, ship_symbol: String) -> Self {
        Self {
            ship_symbol,
            extraction: extraction::ExtractionPilot::new(context.clone()),
            transport: transport::TransportPilot::new(context.clone()),
            survey: surveyor::SurveyPilot::new(context.clone()),
            context,
        }
    }

    #[instrument(level = "info", name = "spacetraders::pilot::mining::pilot_mining", skip(self, pilot, fleet, ship_assignment, mining_config), fields(self.ship_symbol = %self.ship_symbol, fleet_id = fleet.id, ship_assignment_id = ship_assignment.id))]
    pub async fn execute_pilot_circle(
        &self,
        pilot: &super::Pilot,
        fleet: database::Fleet,
        ship_assignment: database::ShipAssignment,
        mining_config: database::MiningFleetConfig,
    ) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        self.assign_ships(ship).await;

        if let ship::AssignmentStatus::Mining { assignment } = &ship.status.status {
            match assignment {
                MiningShipAssignment::Extractor { .. } => {
                    self.run_extractor_ship_worker(ship, pilot).await?
                }
                MiningShipAssignment::Transporter { .. } => {
                    self.run_transporter_ship_worker(ship, pilot).await?
                }
                MiningShipAssignment::Siphoner { .. } => {
                    self.run_siphoned_ship_worker(ship, pilot).await?
                }
                MiningShipAssignment::Surveyor { .. } => {
                    self.run_surveyor_ship_worker(ship, pilot).await?
                }
                MiningShipAssignment::Idle => {}
                MiningShipAssignment::Useless => {}
            }
        }

        Ok(())
    }

    async fn assign_ships(&self, ship: &mut ship::MyShip) {
        ship.status.status = ship::AssignmentStatus::Mining {
            assignment: Self::get_ship_assignment(ship),
        };

        debug!("Assigning role {:?} to ship {}", ship.status, ship.symbol);

        // ship.notify().await;
    }

    pub fn get_ship_assignment(ship: &ship::MyShip) -> MiningShipAssignment {
        let ship_capabilities = Self::analyze_ship_capabilities(ship);

        match ship_capabilities {
            ShipCapabilities {
                can_extract: true,
                can_cargo: true,
                ..
            } => MiningShipAssignment::Extractor {
                state: Default::default(),
                waypoint_symbol: None,
                extractions: None,
            },

            ShipCapabilities {
                can_extract: false,
                can_siphon: true,
                can_cargo: true,
                ..
            } => MiningShipAssignment::Siphoner {
                state: Default::default(),
                waypoint_symbol: None,
                extractions: None,
            },

            ShipCapabilities {
                can_survey: true, ..
            } => MiningShipAssignment::Surveyor {
                surveys: None,
                waypoint_symbol: None,
            },

            ShipCapabilities {
                can_cargo: true, ..
            } => MiningShipAssignment::Transporter {
                state: Default::default(),
                waypoint_symbol: None,
                cycles: None,
            },

            _ => MiningShipAssignment::Useless,
        }
    }

    fn analyze_ship_capabilities(ship: &ship::MyShip) -> ShipCapabilities {
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
        self.survey.execute_survey_circle(ship, pilot).await
    }
}

#[derive(Debug, Clone)]
struct ShipCapabilities {
    can_extract: bool,
    can_siphon: bool,
    can_survey: bool,
    can_cargo: bool,
}
