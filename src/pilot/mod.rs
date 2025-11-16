mod charting;
mod construction;
mod contract;
pub mod mining;
mod scraper;
mod trading;

use charting::ChartPilot;
use construction::ConstructionPilot;
use contract::ContractPilot;
use mining::MiningPilot;
use scraper::ScraperPilot;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use tracing::instrument;
use trading::TradingPilot;
use utils::WaypointCan;

use crate::{
    error::{Error, Result},
    utils::ConductorContext,
};

pub struct Pilot {
    context: ConductorContext,
    ship_symbol: String,
    cancellation_token: CancellationToken,
    construction_pilot: ConstructionPilot,
    trading_pilot: TradingPilot,
    scraper_pilot: ScraperPilot,
    contract_pilot: ContractPilot,
    mining_pilot: MiningPilot,
    chart_pilot: ChartPilot,
}

impl Pilot {
    pub fn new(
        context: ConductorContext,
        ship_symbol: String,
        cancellation_token: CancellationToken,
    ) -> Self {
        debug!(ship_symbol, "Creating pilot for ship");

        Self {
            context: context.clone(),
            ship_symbol: ship_symbol.clone(),
            cancellation_token,
            construction_pilot: ConstructionPilot::new(context.clone(), ship_symbol.clone()),
            trading_pilot: TradingPilot::new(context.clone(), ship_symbol.clone()),
            scraper_pilot: ScraperPilot::new(context.clone(), ship_symbol.clone()),
            contract_pilot: ContractPilot::new(context.clone(), ship_symbol.clone()),
            mining_pilot: MiningPilot::new(context.clone(), ship_symbol.clone()),
            chart_pilot: ChartPilot::new(context.clone(), ship_symbol.clone()),
        }
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    #[instrument(level = "info", name = "spacetraders::pilot::pilot_ship", skip(self), fields(self.ship_symbol = %self.ship_symbol), err(Debug))]
    pub async fn pilot_ship(&self) -> Result<()> {
        {
            let span = tracing::info_span!("spacetraders::pilot::pilot_ship_start", ship_symbol=%self.ship_symbol);
            let _enter = span.enter();
            debug!(ship_symbol = %self.ship_symbol, "Starting pilot for ship");
        }
        tokio::time::sleep(std::time::Duration::from_millis(
            500 + rand::random::<u64>() % 500,
        ))
        .await;
        while !self.cancellation_token.is_cancelled() {
            self.pilot_circle().await?;
        }
        Ok(())
    }

    async fn get_ship_assignment(
        &self,
    ) -> Result<(
        database::ShipInfo,
        Option<(database::ShipAssignment, database::Fleet, bool)>,
    )> {
        let ship_info_res =
            database::ShipInfo::get_by_symbol(&self.context.database_pool, &self.ship_symbol).await;

        let ship_info = ship_info_res?.ok_or(Error::General("Ship not found".to_string()))?;

        if let Some(temp_assignment) = ship_info.temp_assignment_id {
            let assignment = self.get_assignment(temp_assignment).await?;

            if let Some((assignment, fleet)) = assignment {
                debug!(ship_symbol = %self.ship_symbol, assignment = ?assignment, fleet = ?fleet, "Ship has temporary assignment in fleet");
                return Ok((ship_info, Some((assignment, fleet, true))));
            }
        } else if let Some(assignment_id) = ship_info.assignment_id {
            let assignment = self.get_assignment(assignment_id).await?;

            if let Some((assignment, fleet)) = assignment {
                debug!(ship_symbol = %self.ship_symbol, assignment = ?assignment, fleet = ?fleet, "Ship has assignment in fleet");
                return Ok((ship_info, Some((assignment, fleet, false))));
            }
        } else {
            debug!(ship_symbol = %self.ship_symbol, "Ship has no assignment");
        }
        Ok((ship_info, None))
    }

    async fn get_assignment(
        &self,
        assignment_id: i64,
    ) -> Result<Option<(database::ShipAssignment, database::Fleet)>> {
        let assignment =
            database::ShipAssignment::get_by_id(&self.context.database_pool, assignment_id).await?;

        if let Some(assignment) = assignment {
            let fleet =
                database::Fleet::get_by_id(&self.context.database_pool, assignment.fleet_id)
                    .await?
                    .ok_or(crate::error::Error::FleetNotFound {
                        fleet_id: assignment.fleet_id,
                        assignment_id: Some(assignment.id),
                    })?;
            Ok(Some((assignment, fleet)))
        } else {
            Ok(None)
        }
    }

    async fn wait_for_activation(&self) -> Result<()> {
        debug!("Waiting for activation");
        todo!()
    }

    #[instrument(level = "info", name = "spacetraders::pilot::pilot_circle", skip(self), fields(self.ship_symbol = %self.ship_symbol), err(Debug))]
    async fn pilot_circle(&self) -> Result<()> {
        let (ship_info, assignment) = self.get_ship_assignment().await?;

        if !ship_info.active {
            debug!(ship_symbol = %self.ship_symbol, "Ship is inactive, waiting");
            self.wait_for_activation().await?;
            return Ok(());
        }

        if let Some((assignment, fleet, is_temp)) = assignment {
            debug!(ship_symbol = %self.ship_symbol, assignment = ?assignment, fleet = ?fleet, "Piloting ship with assignment in fleet");

            if assignment.disabled {
                debug!(assignment_id = assignment.id, ship_symbol = %self.ship_symbol, "Assignment is disabled, unassigning ship");
                if is_temp {
                    database::ShipInfo::unassign_ship(
                        &self.context.database_pool,
                        &self.ship_symbol,
                    )
                    .await?;
                } else {
                    database::ShipInfo::unassign_temp_ship(
                        &self.context.database_pool,
                        &self.ship_symbol,
                    )
                    .await?;
                }
                return Ok(());
            }

            if !ship_info.active || !fleet.active {
                debug!(ship_symbol = %self.ship_symbol, fleet_id = fleet.id, "Ship or fleet is inactive, waiting");
                self.wait_for_activation().await?;
                return Ok(());
            }

            self.set_ship_assignment(&assignment, &fleet, is_temp)
                .await?;

            self.fly_to_system(&fleet, &assignment).await?;

            match fleet.get_config()? {
                database::FleetConfig::Trading(trading_config) => {
                    self.trading_pilot
                        .execute_pilot_circle(self, fleet, assignment, trading_config)
                        .await?;
                }
                database::FleetConfig::Scraping(scraping_config) => {
                    self.scraper_pilot
                        .execute_pilot_circle(self, fleet, assignment, scraping_config)
                        .await?;
                }
                database::FleetConfig::Mining(mining_config) => {
                    self.mining_pilot
                        .execute_pilot_circle(self, fleet, assignment, mining_config)
                        .await?;
                }
                database::FleetConfig::Charting(charting_config) => {
                    self.chart_pilot
                        .execute_pilot_circle(self, fleet, assignment, charting_config)
                        .await?;
                }
                database::FleetConfig::Construction(construction_config) => {
                    self.construction_pilot
                        .execute_pilot_circle(self, fleet, assignment, construction_config)
                        .await?;
                }
                database::FleetConfig::Contract(contract_config) => {
                    self.contract_pilot
                        .execute_pilot_circle(self, fleet, assignment, contract_config)
                        .await?;
                }
                database::FleetConfig::Manuel(_) => {
                    debug!(fleet_id = fleet.id, ship_symbol = %self.ship_symbol, "Fleet is manuel, ship piloting idle behavior");
                    tokio::time::sleep(std::time::Duration::from_millis(
                        60_000 + rand::random::<u64>() % 1_000,
                    ))
                    .await;
                }
            }

            if is_temp {
                debug!(ship_symbol = %self.ship_symbol, "Clearing temporary assignment for ship");
                database::ShipInfo::unassign_temp_ship(
                    &self.context.database_pool,
                    &self.ship_symbol,
                )
                .await?;
            }
        } else {
            debug!(ship_symbol = %self.ship_symbol, "Ship has no assignment, piloting idle behavior");
            let ship_clone = self.context.ship_manager.get_clone(&ship_info.symbol);
            if let Some(ship_clone) = ship_clone {
                let assignment_id = self
                    .context
                    .fleet_manager
                    .get_new_assignment(&ship_clone)
                    .await?;
                debug!(ship_symbol = %self.ship_symbol, assignment_id = ?assignment_id, "Assigning ship to fleet");
            } else {
                debug!(ship_symbol = %self.ship_symbol, "Ship not found in ship manager, sleeping");
                tokio::time::sleep(std::time::Duration::from_millis(
                    60_000 + rand::random::<u64>() % 1_000,
                ))
                .await;
            }
        }

        Ok(())
    }

    async fn set_ship_assignment(
        &self,
        assignment: &database::ShipAssignment,
        fleet: &database::Fleet,
        is_temp: bool,
    ) -> Result<()> {
        let mut erg = self.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        if is_temp {
            ship.status.temp_assignment_id = Some(assignment.id);
            ship.status.temp_fleet_id = Some(fleet.id);
        } else {
            ship.status.assignment_id = Some(assignment.id);
            ship.status.fleet_id = Some(fleet.id);
        }

        ship.notify(true).await;

        Ok(())
    }

    async fn fly_to_system(
        &self,
        fleet: &database::Fleet,
        assignment: &database::ShipAssignment,
    ) -> crate::error::Result<()> {
        let system = fleet.system_symbol.clone();

        let mut erg = self.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        if ship.nav.system_symbol == system {
            return Ok(());
        }

        self.execute_fly_to_system(ship, fleet, assignment, system)
            .await?;

        Ok(())
    }

    #[instrument(level = "debug", name = "spacetraders::pilot::execute_fly_to_system", skip(self, ship), fields(self.ship_symbol = %self.ship_symbol), err(Debug))]
    async fn execute_fly_to_system(
        &self,
        ship: &mut ship::MyShip,
        fleet: &database::Fleet,
        assignment: &database::ShipAssignment,
        system_symbol: String,
    ) -> crate::error::Result<()> {
        ship.status.status = ship::AssignmentStatus::Transfer {
            fleet_id: fleet.id,
            assignment_id: assignment.id,
            system_symbol: system_symbol.clone(),
        };

        let waypoints =
            database::Waypoint::get_by_system(&self.context.database_pool, &system_symbol).await?;
        let jump_gate = waypoints
            .iter()
            .find(|w| w.is_jump_gate())
            .cloned()
            .ok_or(Error::General("No jump gate found".to_string()))?;

        let budget_manager = self.context.budget_manager.clone();

        let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

        ship.nav_to(
            &jump_gate.symbol,
            true,
            database::TransactionReason::None,
            &self.context.database_pool,
            &self.context.api,
            update_funds_fn,
        )
        .await?;
        Ok(())
    }
}
