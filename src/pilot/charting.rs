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

    #[instrument(level = "info", name = "spacetraders::pilot::charting::pilot_chart", skip(self, pilot, fleet, ship_assignment, charting_config), fields(self.ship_symbol = %self.ship_symbol, chart_waypoint, fleet_id = fleet.id, ship_assignment_id = ship_assignment.id))]
    pub async fn execute_pilot_circle(
        &self,
        pilot: &super::Pilot,
        fleet: database::Fleet,
        ship_assignment: database::ShipAssignment,
        is_temp: bool,
        charting_config: database::ChartingFleetConfig,
    ) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        debug!(ship_symbol = %ship.symbol, "Requesting next chart for ship");

        ship.status.status = ship::AssignmentStatus::Charting {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            waiting_for_manager: true,
            waypoint_symbol: None,
        };
        ship.notify(true).await;

        let chart = self.context.chart_manager.get_next(ship.clone()).await?;

        debug!(chart = ?chart, "Next chart");

        let chart = match chart {
            NextChartResp::Next(chart) => chart,
            NextChartResp::NoChartsInSystem => {
                debug!("No chart available, doing something else");
                return self.do_elsewhere(ship, pilot, is_temp).await;
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

        ship.status.status = ship::AssignmentStatus::Charting {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            waiting_for_manager: false,
            waypoint_symbol: Some(chart.clone()),
        };
        ship.notify(true).await;

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

        ship.status.status = ship::AssignmentStatus::Charting {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            waiting_for_manager: false,
            waypoint_symbol: None,
        };
        ship.notify(true).await;

        Ok(())
    }

    async fn do_elsewhere(
        &self,
        ship: &mut ship::MyShip,
        pilot: &super::Pilot,
        is_temp: bool,
    ) -> std::result::Result<(), Error> {
        // todo: when this happens disable the charting assignments

        if is_temp {
            database::ShipInfo::unassign_ship(&self.context.database_pool, &pilot.ship_symbol)
                .await?;
        } else {
            database::ShipInfo::unassign_temp_ship(&self.context.database_pool, &pilot.ship_symbol)
                .await?;
        }

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

        debug!(waypoint=?sql_waypoint, "Charted Waypoint");

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
                (*jump_gate.data).clone(),
            )
            .await?;

            self.context
                .fleet_manager
                .populate_from_jump_gate(sql_waypoint.symbol.clone())
                .await?;
        }

        let (total_to_chart, marketplace_to_chart) = self
            .get_still_to_chart_in_system(&sql_waypoint.system_symbol)
            .await?;

        // repopulate if it was the last marketplace to chart
        if marketplace_to_chart == 0 && sql_waypoint.is_marketplace() {
            self.context
                .fleet_manager
                .populate_system(sql_waypoint.system_symbol.clone())
                .await?;
        }

        // repopulate if it was the last waypoint to chart
        if total_to_chart == 0 {
            self.context
                .fleet_manager
                .populate_system(sql_waypoint.system_symbol.clone())
                .await?;
        }

        debug!(
            total_to_chart,
            marketplace_to_chart, "Waypoints still to chart in system"
        );

        Ok(())
    }

    /// Returns the number of waypoints still to chart in the given system, first is the total remaining, second are the remaining marketplace waypoints
    async fn get_still_to_chart_in_system(
        &self,
        system_symbol: &str,
    ) -> std::result::Result<(usize, usize), Error> {
        let waypoints_in_system =
            database::Waypoint::get_by_system(&self.context.database_pool, system_symbol).await?;

        let total_to_chart = waypoints_in_system
            .iter()
            .filter(|w| !w.is_charted())
            .count();
        let marketplace_to_chart = waypoints_in_system
            .iter()
            .filter(|w| !w.is_charted() && w.is_marketplace())
            .count();

        Ok((total_to_chart, marketplace_to_chart))
    }
}
