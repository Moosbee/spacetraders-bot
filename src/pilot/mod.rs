mod construction;
mod contract;
mod mining;
mod scraper;
mod trading;

use construction::ConstructionPilot;
use contract::ContractPilot;
use mining::MiningPilot;
use scraper::ScraperPilot;
use tokio_util::sync::CancellationToken;
use trading::TradingPilot;

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
        let context = Self {
            context,
            ship_symbol,
            cancellation_token: CancellationToken::new(),
            construction_pilot: ConstructionPilot::new(),
            trading_pilot: TradingPilot::new(),
            scraper_pilot: ScraperPilot::new(),
            contract_pilot: ContractPilot::new(),
            mining_pilot: MiningPilot::new(),
        };

        context
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub async fn pilot_ship(&self) -> Result<()> {
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
                        _ = tokio::time::sleep(std::time::Duration::from_millis(1_000+ rand::random::<u64>() % 1_000)) => (),
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
                        _ = tokio::time::sleep(std::time::Duration::from_millis(1_000+ rand::random::<u64>() % 1_000)) => (),
            };
        }
        Ok(())
    }

    async fn pilot_circle(&self) -> Result<()> {
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

        let _erg = match ship.role {
            ship::Role::Construction => self.construction_pilot.execute_pilot_circle(&self).await,
            ship::Role::Trader(_) => self.trading_pilot.execute_pilot_circle(&self).await,
            ship::Role::Contract(_) => self.contract_pilot.execute_pilot_circle(&self).await,
            ship::Role::Scraper => self.scraper_pilot.execute_pilot_circle(&self).await,
            ship::Role::Mining(_) => self.mining_pilot.execute_pilot_circle(&self).await,
            ship::Role::Manuel => self.wait_for_new_role().await,
        }?;

        Ok(())
    }
}
