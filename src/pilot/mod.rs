mod charting;
mod construction;
mod contract;
pub mod mining;
mod scraper;
mod trading;
mod transfer;

use charting::ChartPilot;
use construction::ConstructionPilot;
use contract::ContractPilot;
use mining::MiningPilot;
use scraper::ScraperPilot;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use tracing::instrument;
use trading::TradingPilot;
use transfer::TransferPilot;

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
    transfer_pilot: TransferPilot,
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
            transfer_pilot: TransferPilot::new(context.clone(), ship_symbol.clone()),
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
            let ship_info_res =
                database::ShipInfo::get_by_symbol(&self.context.database_pool, &self.ship_symbol)
                    .await;

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
                database::ShipInfo::get_by_symbol(&self.context.database_pool, &self.ship_symbol)
                    .await?
                    .ok_or(Error::General("Ship not found".to_string()))?;

            if ship_info.active {
                break;
            }
            tokio::select! {
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
                database::ShipInfo::get_by_symbol(&self.context.database_pool, &self.ship_symbol)
                    .await?
                    .ok_or(Error::General("Ship not found".to_string()))?;

            if ship_info.role != database::ShipInfoRole::Manuel {
                break;
            }

            tokio::select! {
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

            ship.notify().await;

            if !ship.active {
                return Ok(());
            }
            ship.role
        };

        debug!("Starting pilot circle for ship {}", self.ship_symbol);

        match role {
            database::ShipInfoRole::Construction => {
                self.construction_pilot.execute_pilot_circle(self).await
            }
            database::ShipInfoRole::Trader => self.trading_pilot.execute_pilot_circle(self).await,
            database::ShipInfoRole::Contract => {
                self.contract_pilot.execute_pilot_circle(self).await
            }
            database::ShipInfoRole::Scraper => self.scraper_pilot.execute_pilot_circle(self).await,
            database::ShipInfoRole::Mining => self.mining_pilot.execute_pilot_circle(self).await,
            database::ShipInfoRole::Manuel => self.wait_for_new_role().await,
            database::ShipInfoRole::TempTrader => {
                self.trading_pilot.execute_pilot_circle(self).await
            }
            database::ShipInfoRole::Charter => self.chart_pilot.execute_pilot_circle(self).await,
            database::ShipInfoRole::Transfer => {
                self.transfer_pilot.execute_pilot_circle(self).await
            }
        }?;

        Ok(())
    }

    async fn get_budget(&self) -> Result<i64> {
        let agent_symbol = { self.context.run_info.read().await.agent_symbol.clone() };

        let agent = database::Agent::get_last_by_symbol(&self.context.database_pool, &agent_symbol)
            .await?
            .ok_or(Error::General("Agent not found".to_string()))?;
        Ok(agent.credits - 30_000)
    }
}
