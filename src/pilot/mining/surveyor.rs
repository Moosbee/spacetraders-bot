use std::{
    collections::HashMap,
    sync::{atomic::AtomicI32, Arc},
};

use database::DatabaseConnector;
use ship::status::MiningShipAssignment;
use tracing::instrument;

use crate::{error::Result, utils::ConductorContext};

pub struct SurveyPilot {
    count: Arc<AtomicI32>,
    context: ConductorContext,
}

impl SurveyPilot {
    pub fn new(context: ConductorContext) -> Self {
        Self {
            count: Arc::new(AtomicI32::new(0)),
            context,
        }
    }

    #[instrument(level = "info", name = "spacetraders::pilot::pilot_survey", skip(self, pilot, ship), fields(self.ship_symbol = pilot.ship_symbol, waypoint))]
    pub async fn execute_survey_circle(
        &self,
        ship: &mut ship::MyShip,
        pilot: &crate::pilot::Pilot,
    ) -> Result<()> {
        let waypoint = self.get_waypoint(ship).await;

        if waypoint.is_none() {
            tokio::time::sleep(tokio::time::Duration::from_millis(1_000)).await;
            return Ok(());
        }

        let waypoint = waypoint.unwrap();

        tracing::Span::current().record("waypoint", &waypoint);

        ship.status = ship::ShipStatus::Mining {
            assignment: MiningShipAssignment::Surveyor {
                waypoint_symbol: Some(waypoint.clone()),
                surveys: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            },
        };

        ship.notify().await;

        let budget_manager = self.context.budget_manager.clone();

        let update_funds_fn = move |amount| budget_manager.set_current_funds(amount);

        ship.nav_to(
            &waypoint,
            true,
            database::TransactionReason::MiningWaypoint(waypoint.clone()),
            &self.context.database_pool,
            &self.context.api,
            update_funds_fn,
        )
        .await?;

        ship.wait_for_arrival().await;

        ship.wait_for_cooldown().await;

        let ship_before = ship.snapshot(&self.context.database_pool).await?;

        let surveys = ship.survey(&self.context.api).await?;

        let ship_after = ship.snapshot(&self.context.database_pool).await?;

        let all_surveys = surveys
            .data
            .surveys
            .into_iter()
            .map(|f| database::Survey::from_model(f, ship_before, ship_after))
            .collect::<database::Result<Vec<_>, _>>()?;

        database::Survey::insert_bulk(&self.context.database_pool, &all_surveys).await?;

        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        ship.status = ship::ShipStatus::Mining {
            assignment: MiningShipAssignment::Surveyor {
                waypoint_symbol: None,
                surveys: Some(self.count.load(std::sync::atomic::Ordering::Relaxed)),
            },
        };

        ship.notify().await;
        Ok(())
    }

    async fn get_waypoint(&self, ship: &mut ship::MyShip) -> Option<String> {
        let all_system_ships = self
            .context
            .ship_manager
            .get_all_clone()
            .await
            .into_values()
            .filter(|f| f.nav.system_symbol == ship.nav.system_symbol)
            .filter(|f| matches!(&f.status, ship::ShipStatus::Mining { .. }))
            .collect::<Vec<_>>();

        let surveyors = all_system_ships
            .iter()
            .filter_map(|f| match &f.status {
                ship::ShipStatus::Mining {
                    assignment:
                        MiningShipAssignment::Surveyor {
                            waypoint_symbol, ..
                        },
                } => waypoint_symbol.as_ref(),
                _ => None,
            })
            .collect::<Vec<_>>();

        let miners = all_system_ships
            .iter()
            .filter_map(|f| match &f.status {
                ship::ShipStatus::Mining {
                    assignment:
                        MiningShipAssignment::Extractor {
                            waypoint_symbol, ..
                        },
                } => waypoint_symbol.as_ref(),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut waypoints = HashMap::new();

        for miner in miners {
            waypoints.entry(miner).or_insert((0, 0)).0 += 1;
        }

        for surveyor in surveyors {
            waypoints.entry(surveyor).or_insert((0, 0)).1 += 1;
        }

        let wps = waypoints
            .iter()
            .max_by(|s1, s2| s1.1 .1.cmp(&s2.1 .1).then(s1.1 .0.cmp(&s2.1 .0)));

        wps.map(|f| (**f.0).clone())
    }
}
