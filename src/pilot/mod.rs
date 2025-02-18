mod construction;
mod contract;
mod mining;
mod scraper;
mod trading;

use construction::ConstructionPilot;
use contract::ContractPilot;
use log::debug;
use mining::MiningPilot;
use scraper::ScraperPilot;
use tokio_util::sync::CancellationToken;
use trading::TradingPilot;

use crate::config::CONFIG;
use crate::{ship, sql, workers::types::ConductorContext};

use crate::error::{Error, Result};

pub struct Pilot {
    context: ConductorContext,
    ship_symbol: String,
    cancellation_token: CancellationToken,
    construction_pilot: ConstructionPilot,
    trading_pilot: TradingPilot,
    scraper_pilot: ScraperPilot,
    contract_pilot: ContractPilot,
    mining_pilot: MiningPilot,
}

impl Pilot {
    pub fn new(context: ConductorContext, ship_symbol: String) -> Self {
        debug!("Creating pilot for ship {}", ship_symbol);
        let pilot = Self {
            context: context.clone(),
            ship_symbol: ship_symbol.clone(),
            cancellation_token: CancellationToken::new(),
            construction_pilot: ConstructionPilot::new(),
            trading_pilot: TradingPilot::new(context.clone(), ship_symbol.clone()),
            scraper_pilot: ScraperPilot::new(),
            contract_pilot: ContractPilot::new(context.clone(), ship_symbol.clone()),
            mining_pilot: MiningPilot::new(context.clone(), ship_symbol.clone()),
        };

        pilot
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub async fn pilot_ship(&self) -> Result<()> {
        debug!("Starting pilot for ship {}", self.ship_symbol);
        while !self.cancellation_token.is_cancelled() {
            let ship_info_res =
                sql::ShipInfo::get_by_symbol(&self.context.database_pool, &self.ship_symbol).await;

            let ship_info = ship_info_res?.ok_or(Error::General("Ship not found".to_string()))?;

            if !ship_info.active {
                self.wait_for_activation().await?;
            }

            self.pilot_circle().await?;
        }
        Ok(())
    }

    async fn wait_for_activation(&self) -> Result<()> {
        debug!("Waiting for activation");
        while !self.cancellation_token.is_cancelled() {
            let ship_info =
                sql::ShipInfo::get_by_symbol(&self.context.database_pool, &self.ship_symbol)
                    .await?
                    .ok_or(Error::General("Ship not found".to_string()))?;

            if ship_info.active {
                break;
            }
            let _ = tokio::select! {
                        _ = self.cancellation_token.cancelled() => {
                            return Ok(());
                        },
                        _ = tokio::time::sleep(std::time::Duration::from_millis(10_000+ rand::random::<u64>() % 1_000)) => (),
            };
        }
        Ok(())
    }

    async fn wait_for_new_role(&self) -> Result<()> {
        while !self.cancellation_token.is_cancelled() {
            let ship_info =
                sql::ShipInfo::get_by_symbol(&self.context.database_pool, &self.ship_symbol)
                    .await?
                    .ok_or(Error::General("Ship not found".to_string()))?;

            if ship_info.role != sql::ShipInfoRole::Manuel {
                break;
            }
            let _ = tokio::select! {
                        _ = self.cancellation_token.cancelled() => {
                            return Ok(());
                        },
                        _ = tokio::time::sleep(std::time::Duration::from_millis(10_000+ rand::random::<u64>() % 1_000)) => (),
            };
        }
        Ok(())
    }

    async fn pilot_circle(&self) -> Result<()> {
        let role = {
            let mut ship_guard = self.context.ship_manager.get_mut(&self.ship_symbol).await;
            // .ok_or(Error::General("Ship was locked".to_string()))?;
            let ship = ship_guard
                .value_mut()
                .ok_or(Error::General("Ship not found".to_string()))?;

            ship.apply_from_db(self.context.database_pool.clone())
                .await?;

            if !ship.active {
                return Ok(());
            }
            ship.role.clone()
        };

        debug!("Starting pilot circle for ship {}", self.ship_symbol);

        let _erg = match role {
            sql::ShipInfoRole::Construction => {
                self.construction_pilot.execute_pilot_circle(&self).await
            }
            sql::ShipInfoRole::Trader => self.trading_pilot.execute_pilot_circle(&self).await,
            sql::ShipInfoRole::Contract => self.contract_pilot.execute_pilot_circle(&self).await,
            sql::ShipInfoRole::Scraper => self.scraper_pilot.execute_pilot_circle(&self).await,
            sql::ShipInfoRole::Mining => self.mining_pilot.execute_pilot_circle(&self).await,
            sql::ShipInfoRole::Manuel => self.wait_for_new_role().await,
        }?;

        Ok(())
    }

    async fn get_budget(&self) -> Result<i64> {
        let agent =
            sql::Agent::get_last_by_symbol(&self.context.database_pool, &CONFIG.symbol).await?;
        Ok(agent.credits - 30_000)
    }
}
