use std::sync::{atomic::AtomicI32, Arc};

use database::DatabaseConnector;
use tracing::debug;
use tracing::instrument;
use utils::WaypointCan;

use crate::{
    error::{Error, Result},
    manager::chart_manager::NextChartResp,
    utils::ConductorContext,
};
pub struct ChartPilot {
    context: ConductorContext,
    ship_symbol: String,
    count: Arc<AtomicI32>,
}

impl ChartPilot {
    pub fn new(context: ConductorContext, ship_symbol: String) -> Self {
        Self {
            context,
            ship_symbol,
            count: Arc::new(AtomicI32::new(0)),
        }
    }

    #[instrument(level = "info", name = "spacetraders::pilot::pilot_chart", skip(self, pilot), fields(self.ship_symbol = %self.ship_symbol, chart_waypoint))]
    pub async fn execute_pilot_circle(&self, pilot: &super::Pilot) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        debug!("Requesting next chart for ship: {:?}", ship.symbol);

        ship.status = ship::ShipStatus::Charting {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            waiting_for_manager: true,
            waypoint_symbol: None,
        };
        ship.notify().await;

        let chart = self.context.chart_manager.get_next(ship.clone()).await?;

        debug!("Next chart: {:?}", chart);

        let chart = match chart {
            NextChartResp::Next(chart) => chart,
            NextChartResp::NoChartsInSystem => {
                debug!("No chart available, doing something else");
                return self.do_elsewhere(ship).await;
            }
        };

        tracing::Span::current().record("chart_waypoint", &chart);

        let erg = self.start_chart(ship, chart.clone()).await;

        if let Err(e) = erg {
            self.context.chart_manager.fail_chart(chart.clone()).await?;
            return Err(e);
        }

        Ok(())
    }

    async fn start_chart(
        &self,
        ship: &mut ship::MyShip,
        chart: String,
    ) -> std::result::Result<(), Error> {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        ship.status = ship::ShipStatus::Charting {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            waiting_for_manager: false,
            waypoint_symbol: Some(chart.clone()),
        };
        ship.notify().await;

        let budget_manager = self.context.budget_manager.clone();

        let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

        ship.nav_to(
            &chart,
            true,
            database::TransactionReason::None,
            &self.context.database_pool,
            &self.context.api,
            update_funds_fn,
        )
        .await?;

        self.chart_waypoint(ship).await?;

        self.context.chart_manager.complete_chart(chart).await?;

        ship.status = ship::ShipStatus::Charting {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            waiting_for_manager: false,
            waypoint_symbol: None,
        };
        ship.notify().await;

        Ok(())
    }

    async fn do_elsewhere(&self, ship: &mut ship::MyShip) -> std::result::Result<(), Error> {
        ship.status = ship::ShipStatus::Manuel;
        let role = if ship.cargo.capacity == 0 {
            database::ShipInfoRole::Scraper
        } else {
            database::ShipInfoRole::Trader
        };

        let sql_ship =
            database::ShipInfo::get_by_symbol(&self.context.database_pool, &ship.symbol).await?;
        if let Some(mut sql_ship) = sql_ship {
            sql_ship.role = role;
            database::ShipInfo::insert(&self.context.database_pool, &sql_ship).await?;
        }

        ship.apply_from_db(self.context.database_pool.clone())
            .await?;

        debug!("Doing something else");
        ship.notify().await;
        Ok(())
    }

    async fn chart_waypoint(&self, ship: &mut ship::MyShip) -> std::result::Result<(), Error> {
        let symbol = ship.symbol.clone();

        let erg = self.context.api.create_chart(&symbol).await;

        let (waypoint, agent_info) = match erg {
            Ok(data) => (
                *data.data.waypoint,
                Some((*data.data.agent, *data.data.transaction)),
            ),
            Err(space_traders_client::apis::Error::ResponseError(error)) => {
                if error
                    .entity
                    .as_ref()
                    .map(|e| {
                        e.error.code
                            == space_traders_client::models::error_codes::WAYPOINT_CHARTED_ERROR
                    })
                    .unwrap_or(false)
                {
                    (
                        *self
                            .context
                            .api
                            .get_waypoint(&ship.nav.system_symbol, &ship.nav.waypoint_symbol)
                            .await?
                            .data,
                        None,
                    )
                } else {
                    return Err(space_traders_client::apis::Error::ResponseError(error).into());
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        if let Some((agent, transaction)) = agent_info {
            let sql_agent = database::Agent::from(agent);

            self.context
                .budget_manager
                .set_current_funds(sql_agent.credits);

            database::Agent::insert(&self.context.database_pool, &sql_agent).await?;

            let sql_transaction = database::ChartTransaction::try_from(transaction)?;
            database::ChartTransaction::insert(&self.context.database_pool, &sql_transaction)
                .await?;
        }

        let sql_waypoint = (&waypoint).into();

        database::Waypoint::insert(&self.context.database_pool, &sql_waypoint).await?;

        if sql_waypoint.is_marketplace() {
            let market = self
                .context
                .api
                .get_market(&sql_waypoint.system_symbol, &sql_waypoint.symbol)
                .await?;

            crate::manager::scrapping_manager::utils::update_market(
                *market.data,
                &self.context.database_pool,
            )
            .await;
        }

        if sql_waypoint.is_shipyard() {
            let shipyard = self
                .context
                .api
                .get_shipyard(&sql_waypoint.system_symbol, &sql_waypoint.symbol)
                .await?;

            crate::manager::scrapping_manager::utils::update_shipyard(
                &self.context.database_pool,
                *shipyard.data,
            )
            .await?
        }

        if sql_waypoint.is_jump_gate() {
            let jump_gate = self
                .context
                .api
                .get_jump_gate(&sql_waypoint.system_symbol, &sql_waypoint.symbol)
                .await?;

            crate::manager::scrapping_manager::utils::update_jump_gate(
                &self.context.database_pool,
                *jump_gate.data,
            )
            .await?
        }

        Ok(())
    }
}
