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

            // if ship_info.role != database::ShipInfoRole::Manuel {
            //     break;
            // }

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
        todo!();

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
