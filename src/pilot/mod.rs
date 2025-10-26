mod charting;
mod construction;
mod contract;
pub mod mining;
mod scraper;
mod trading;

use charting::ChartPilot;
use construction::ConstructionPilot;
use contract::ContractPilot;
use database::DatabaseConnector;
use mining::MiningPilot;
use scraper::ScraperPilot;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use tracing::instrument;
use trading::TradingPilot;

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
        debug!("Creating pilot for ship {}", ship_symbol);

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
            debug!("Starting pilot for ship {}", self.ship_symbol);
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

    async fn unassign_ship(&self) -> Result<()> {
        let mut ship_info =
            database::ShipInfo::get_by_symbol(&self.context.database_pool, &self.ship_symbol)
                .await?
                .ok_or(Error::General("Ship not found".to_string()))?;

        ship_info.assignment_id = None;

        database::ShipInfo::insert(&self.context.database_pool, &ship_info).await?;

        Ok(())
    }

    async fn unassign_temp_ship(&self) -> Result<()> {
        let mut ship_info =
            database::ShipInfo::get_by_symbol(&self.context.database_pool, &self.ship_symbol)
                .await?
                .ok_or(Error::General("Ship not found".to_string()))?;

        ship_info.temp_assignment_id = None;

        database::ShipInfo::insert(&self.context.database_pool, &ship_info).await?;

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
                debug!(
                    "Ship {} has temporary assignment {:?} in fleet {:?}",
                    self.ship_symbol, assignment, fleet
                );
                return Ok((ship_info, Some((assignment, fleet, true))));
            }
        } else if let Some(assignment_id) = ship_info.assignment_id {
            let assignment = self.get_assignment(assignment_id).await?;

            if let Some((assignment, fleet)) = assignment {
                debug!(
                    "Ship {} has assignment {:?} in fleet {:?}",
                    self.ship_symbol, assignment, fleet
                );
                return Ok((ship_info, Some((assignment, fleet, false))));
            }
        } else {
            debug!("Ship {} has no assignment", self.ship_symbol);
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

        if let Some((assignment, fleet, is_temp)) = assignment {
            debug!(
                "Piloting ship {} with assignment {:?} in fleet {:?}",
                self.ship_symbol, assignment, fleet
            );

            if assignment.disabled {
                debug!(
                    "Assignment {} is disabled, unassigning ship {}",
                    assignment.id, self.ship_symbol
                );
                if is_temp {
                    self.unassign_temp_ship().await?;
                } else {
                    self.unassign_ship().await?;
                }
                return Ok(());
            }

            if !ship_info.active || !fleet.active {
                debug!(
                    "Ship {} or fleet {} is inactive, waiting",
                    self.ship_symbol, fleet.id
                );
                self.wait_for_activation().await?;
                return Ok(());
            }

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
                database::FleetConfig::Manuel => {
                    debug!(
                        "Fleet {} is manuel, ship {} piloting idle behavior",
                        fleet.id, self.ship_symbol
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(
                        60_000 + rand::random::<u64>() % 1_000,
                    ))
                    .await;
                }
            }

            if is_temp {
                debug!(
                    "Clearing temporary assignment for ship {}",
                    self.ship_symbol
                );
                self.unassign_temp_ship().await?;
            }
        } else {
            debug!(
                "Ship {} has no assignment, piloting idle behavior",
                self.ship_symbol
            );
            let ship_clone = self.context.ship_manager.get_clone(&ship_info.symbol);
            if let Some(ship_clone) = ship_clone {
                self.context
                    .fleet_manager
                    .get_new_assignment(&ship_clone)
                    .await?;
            } else {
                debug!(
                    "Ship {} not found in ship manager, sleeping",
                    self.ship_symbol
                );
                tokio::time::sleep(std::time::Duration::from_millis(
                    60_000 + rand::random::<u64>() % 1_000,
                ))
                .await;
            }
        }

        Ok(())
    }
}
