use std::sync::{atomic::AtomicI32, Arc};

use chrono::Utc;
use tracing::debug;
use tracing::instrument;
use utils::WaypointCan;

use crate::{
    error::{Error, Result},
    utils::ConductorContext,
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

    #[instrument(level = "info", name = "spacetraders::pilot::scraper::pilot_scraper", skip(self, pilot, fleet, ship_assignment, scraping_config), fields(self.ship_symbol = %self.ship_symbol, scrap_waypoint = tracing::field::Empty, scrap_date = tracing::field::Empty, fleet_id = fleet.id, ship_assignment_id = ship_assignment.id))]
    pub async fn execute_pilot_circle(
        &self,
        pilot: &super::Pilot,
        fleet: database::Fleet,
        ship_assignment: database::ShipAssignment,
        scraping_config: database::ScrapingFleetConfig,
    ) -> Result<()> {
        let mut erg = pilot.context.ship_manager.get_mut(&self.ship_symbol).await;
        let ship = erg
            .value_mut()
            .ok_or(Error::General("Ship not found".to_string()))?;

        debug!("Requesting next scrap for ship: {:?}", ship.symbol);
        ship.status = ship::ShipStatus::Scraper {
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

        tracing::Span::current().record("scrap_waypoint", &waypoint_symbol);
        tracing::Span::current().record("scrap_date", date.to_rfc3339());

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
        ship: &mut ship::MyShip,
        waypoint_symbol: String,
        date: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        ship.status = ship::ShipStatus::Scraper {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            waiting_for_manager: false,
            waypoint_symbol: Some(waypoint_symbol.clone()),
            scrap_date: Some(date),
        };

        ship.notify().await;

        if waypoint_symbol != ship.nav.waypoint_symbol {
            let budget_manager = self.context.budget_manager.clone();

            let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

            ship.nav_to(
                &waypoint_symbol,
                true,
                database::TransactionReason::None,
                &self.context.database_pool,
                &self.context.api,
                update_funds_fn,
            )
            .await?;
        }

        ship.wait_for_arrival().await;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let waypoint =
            database::Waypoint::get_by_symbol(&self.context.database_pool, &waypoint_symbol)
                .await?
                .ok_or(Error::General("Waypoint not found".to_owned()))?;

        if waypoint.is_shipyard() {
            self.context
                .fleet_manager
                .at_shipyard(ship.nav.waypoint_symbol.clone(), ship.symbol.clone())
                .await?;
        }

        let state = self.wait_until(date).await?;

        if state == 0 {
            self.context
                .scrapping_manager
                .fail(ship.clone(), waypoint_symbol)
                .await?;

            return Ok(());
        }

        let waypoint =
            database::Waypoint::get_by_symbol(&self.context.database_pool, &waypoint_symbol)
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

        ship.status = ship::ShipStatus::Scraper {
            cycle: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            waiting_for_manager: false,
            waypoint_symbol: None,
            scrap_date: None,
        };

        Ok(())
    }

    async fn do_elsewhere(&self, ship: &mut ship::MyShip) -> std::result::Result<(), Error> {
        let temp_assignment = self
            .context
            .fleet_manager
            .get_new_temp_assignment(ship)
            .await?;
        if temp_assignment.is_none() {
            tracing::warn!("No temp assignment available, skipping");
            return Ok(());
        }

        Ok(())
    }

    async fn wait_until(
        &self,
        // ship: &mut ship::MyShip, // at some point for activation and deactivation
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
