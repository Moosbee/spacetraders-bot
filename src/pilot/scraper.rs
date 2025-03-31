use std::{
    collections::HashMap,
    sync::{atomic::AtomicI32, Arc},
};

use chrono::Utc;
use log::debug;

use crate::{
    error::{Error, Result},
    sql::{self, DatabaseConnector},
    types::{ConductorContext, WaypointCan},
};

pub struct ScraperPilot {
    context: ConductorContext,
    ship_symbol: String,
    count: Arc<AtomicI32>,
}

impl ScraperPilot {
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

        debug!("Requesting next scrap for ship: {:?}", ship.symbol);
        ship.status = crate::ship::ShipStatus::Scraper {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            waiting_for_manager: true,
            waypoint_symbol: None,
            scrap_date: None,
        };

        ship.notify().await;

        let scrap = self
            .context
            .scrapping_manager
            .get_next(ship.clone())
            .await?;

        let (waypoint_symbol, date) = match scrap {
            crate::manager::scrapping_manager::ScrapResponse::Unassigned => {
                debug!("Nothing to scrap available, doing something else");
                return self.do_elsewhere(ship).await;
            }
            crate::manager::scrapping_manager::ScrapResponse::Scrapping {
                waypoint_symbol,
                date,
            } => (waypoint_symbol, date),
        };

        let erg = self.scrap(ship, waypoint_symbol.clone(), date).await;

        if let Err(err) = erg {
            self.context
                .scrapping_manager
                .fail(ship.clone(), waypoint_symbol)
                .await?;
            return Err(err);
        }

        Ok(())
    }

    async fn scrap(
        &self,
        ship: &mut crate::ship::MyShip,
        waypoint_symbol: String,
        date: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        ship.status = crate::ship::ShipStatus::Scraper {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            waiting_for_manager: false,
            waypoint_symbol: Some(waypoint_symbol.clone()),
            scrap_date: Some(date),
        };

        ship.notify().await;

        if waypoint_symbol != ship.nav.waypoint_symbol {
            let waypoints =
                sql::Waypoint::get_by_system(&self.context.database_pool, &ship.nav.system_symbol)
                    .await?
                    .into_iter()
                    .map(|w| (w.symbol.clone(), w))
                    .collect::<HashMap<_, _>>();

            ship.nav_to(
                &waypoint_symbol,
                true,
                sql::TransactionReason::None,
                &self.context,
            )
            .await?;
        }

        ship.wait_for_arrival_mut(&self.context.api)
            .await
            .map_err(|e| e.to_string())?;

        let state = self.wait_until(date).await?;

        if state == 0 {
            self.context
                .scrapping_manager
                .fail(ship.clone(), waypoint_symbol)
                .await?;

            return Ok(());
        }

        let waypoint = sql::Waypoint::get_by_symbol(&self.context.database_pool, &waypoint_symbol)
            .await?
            .ok_or(Error::General("Waypoint not found".to_owned()))?;

        if waypoint.is_marketplace() {
            let market_resp = self
                .context
                .api
                .get_market(&ship.nav.system_symbol, &ship.nav.waypoint_symbol)
                .await?;
            crate::manager::scrapping_manager::utils::update_market(
                *market_resp.data,
                &self.context.database_pool,
            )
            .await;
        }
        if waypoint.is_shipyard() {
            let shipyard_resp = self
                .context
                .api
                .get_shipyard(&ship.nav.system_symbol, &ship.nav.waypoint_symbol)
                .await?;

            crate::manager::scrapping_manager::utils::update_shipyard(
                &self.context.database_pool,
                *shipyard_resp.data,
            )
            .await?;
        }

        self.context
            .scrapping_manager
            .complete(ship.clone(), waypoint_symbol)
            .await?;

        ship.status = crate::ship::ShipStatus::Scraper {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            waiting_for_manager: false,
            waypoint_symbol: None,
            scrap_date: None,
        };

        Ok(())
    }

    async fn do_elsewhere(&self, ship: &mut crate::ship::MyShip) -> std::result::Result<(), Error> {
        ship.status = crate::ship::ShipStatus::Manuel;

        let sql_ship =
            sql::ShipInfo::get_by_symbol(&self.context.database_pool, &ship.symbol).await?;
        if let Some(mut sql_ship) = sql_ship {
            sql_ship.role = crate::sql::ShipInfoRole::Manuel;
            sql::ShipInfo::insert(&self.context.database_pool, &sql_ship).await?;
        }

        ship.apply_from_db(self.context.database_pool.clone())
            .await?;

        debug!("Doing something else");
        ship.notify().await;
        Ok(())
    }

    async fn wait_until(
        &self,
        // ship: &mut crate::ship::MyShip, // at some point for activation and deactivation
        date: chrono::DateTime<chrono::Utc>,
    ) -> Result<u32> {
        let t = date - Utc::now();
        let t = t.num_milliseconds();
        if t < 0 {
            return Ok(1);
        }
        let t = t as u64;
        tokio::time::sleep(tokio::time::Duration::from_millis(t)).await;
        Ok(1)
    }
}
