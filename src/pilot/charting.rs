use std::{
    collections::HashMap,
    sync::{atomic::AtomicI32, Arc},
};

use log::debug;

use crate::{
    error::{Error, Result},
    manager::chart_manager::NextChartResp,
    sql::{self, DatabaseConnector},
    types::{ConductorContext, WaypointCan},
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
    pub async fn execute_pilot_circle(&self, pilot: &super::Pilot) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        debug!("Requesting next chart for ship: {:?}", ship.symbol);

        ship.status = crate::ship::ShipStatus::Charting {
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

        let erg = self.start_chart(ship, chart.clone()).await;

        if let Err(e) = erg {
            self.context.chart_manager.fail_chart(chart.clone()).await?;
            return Err(e);
        }

        Ok(())
    }

    async fn start_chart(
        &self,
        ship: &mut crate::ship::MyShip,
        chart: String,
    ) -> std::result::Result<(), Error> {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        ship.status = crate::ship::ShipStatus::Charting {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            waiting_for_manager: false,
            waypoint_symbol: Some(chart.clone()),
        };
        ship.notify().await;

        let waypoints =
            sql::Waypoint::get_by_system(&self.context.database_pool, &ship.nav.system_symbol)
                .await?
                .into_iter()
                .map(|w| (w.symbol.clone(), w))
                .collect::<HashMap<_, _>>();

        ship.nav_to(
            &chart,
            true,
            &waypoints,
            &self.context.api,
            self.context.database_pool.clone(),
            sql::TransactionReason::None,
        )
        .await?;

        self.chart_waypoint(ship).await?;

        self.context.chart_manager.complete_chart(chart).await?;

        ship.status = crate::ship::ShipStatus::Charting {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::SeqCst)),
            waiting_for_manager: false,
            waypoint_symbol: None,
        };
        ship.notify().await;

        Ok(())
    }

    async fn do_elsewhere(&self, ship: &mut crate::ship::MyShip) -> std::result::Result<(), Error> {
        ship.status = crate::ship::ShipStatus::Manuel;
        let role = if ship.cargo.capacity == 0 {
            crate::sql::ShipInfoRole::Scraper
        } else {
            crate::sql::ShipInfoRole::Trader
        };

        let sql_ship =
            sql::ShipInfo::get_by_symbol(&self.context.database_pool, &ship.symbol).await?;
        if let Some(mut sql_ship) = sql_ship {
            sql_ship.role = role;
            sql::ShipInfo::insert(&self.context.database_pool, &sql_ship).await?;
        }

        ship.apply_from_db(self.context.database_pool.clone())
            .await?;

        debug!("Doing something else");
        ship.notify().await;
        Ok(())
    }

    async fn chart_waypoint(
        &self,
        ship: &mut crate::ship::MyShip,
    ) -> std::result::Result<(), Error> {
        let symbol = ship.symbol.clone();

        let erg = self.context.api.create_chart(&symbol).await;

        let waypoint = match erg {
            Ok(data) => *data.data.waypoint,
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
                    *self
                        .context
                        .api
                        .get_waypoint(&ship.nav.system_symbol, &ship.nav.waypoint_symbol)
                        .await?
                        .data
                } else {
                    return Err(space_traders_client::apis::Error::ResponseError(error).into());
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        let sql_waypoint = (&waypoint).into();

        sql::Waypoint::insert(&self.context.database_pool, &sql_waypoint).await?;

        if sql_waypoint.is_marketplace() {
            let market = self
                .context
                .api
                .get_market(&sql_waypoint.system_symbol, &sql_waypoint.symbol)
                .await?;

            crate::manager::scrapping_manager::update_market(
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

            crate::manager::scrapping_manager::update_shipyard(
                &self.context.database_pool,
                *shipyard.data,
            )
            .await?
        }

        Ok(())
    }
}
