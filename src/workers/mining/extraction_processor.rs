use std::sync::Arc;

use anyhow::Ok;
use futures::StreamExt;
use log::{debug, info};
use space_traders_client::models;
use tokio_util::sync::CancellationToken;

use crate::{
    config::CONFIG,
    ship,
    sql::TransactionReason,
    types::{safely_get_map, ConductorContext, WaypointCan},
};

use super::mining_manager::MiningManager;

#[derive(Debug, Clone)]
pub struct ExtractionProcessor {
    context: ConductorContext,
    cancellation_token: CancellationToken,
    mining_places: Arc<MiningManager>,
}

impl ExtractionProcessor {
    pub fn new(
        context: ConductorContext,
        cancellation_token: CancellationToken,
        mining_places: Arc<MiningManager>,
    ) -> ExtractionProcessor {
        ExtractionProcessor {
            context,
            cancellation_token,
            mining_places,
        }
    }

    /// Generic method to handle both extraction and siphoning workflows
    pub async fn run_ship_worker(
        &self,
        ship: &mut ship::MyShip,
        is_siphon: bool,
    ) -> anyhow::Result<()> {
        // Random small delay to distribute load
        tokio::time::sleep(std::time::Duration::from_millis(
            100 + rand::random::<u64>() % 1000,
        ))
        .await;

        let current_waypoint = self.get_current_waypoint(ship);
        let mut wp = None;

        // Determine if current waypoint is suitable
        let mut worked = if is_siphon && current_waypoint.is_sipherable()
            || !is_siphon && current_waypoint.is_minable()
        {
            self.mining_places
                .assign_to(&ship.symbol, &current_waypoint)
                .await
        } else {
            false
        };

        if worked {
            wp = Some(current_waypoint);
        }

        // Find alternative waypoints if current one doesn't work
        while !worked {
            ship.sleep(
                std::time::Duration::from_millis(100 + rand::random::<u64>() % 1000),
                &self.context.api,
            )
            .await;

            let waypoints = self
                .find_suitable_waypoints(ship.nav.system_symbol.clone(), is_siphon)
                .await;

            if let Some(waypoint) = waypoints.first() {
                worked = self
                    .mining_places
                    .assign_to(&ship.symbol, &waypoint.0)
                    .await;
                let ship_resp = self.context.api.get_my_ship(&ship.symbol).await?;
                ship.update(*ship_resp.data);
                if worked {
                    wp = Some(waypoint.0.clone());
                }
            }
        }

        // Navigate and perform extraction/siphoning
        if let Some(waypoint) = wp {
            let system_symbol = waypoint.system_symbol.clone();
            let waypoints = safely_get_map(&self.context.all_waypoints, &system_symbol)
                .unwrap()
                .clone();

            ship.nav_to(
                &waypoint.symbol,
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                TransactionReason::MiningWaypoint(waypoint.symbol.clone()),
            )
            .await?;

            self.run_cycle(ship, &waypoint, is_siphon).await?
        }

        Ok(())
    }

    /// Simplified extraction cycle
    async fn run_cycle(
        &self,
        ship: &mut ship::MyShip,
        _waypoint: &models::Waypoint,
        is_siphon: bool,
    ) -> anyhow::Result<()> {
        for _ in 0..CONFIG.mining.max_extractions_per_miner {
            let i = tokio::select! {
                _ = self.cancellation_token.cancelled() => {0},// it's the end of the Programm we don't care(for now)
                _ = ship.wait_for_cooldown_mut(&self.context.api) => {1},
            };

            if i == 0 {
                info!("Cycle cancelled for {}", ship.symbol);
                break;
            }

            // debug!("Extraction cycle");

            if ship.cargo.units >= ship.cargo.capacity {
                ship.sleep(
                    std::time::Duration::from_millis(2000 + rand::random::<u64>() % 5000),
                    &self.context.api,
                )
                .await;
                continue;
            }

            debug!("Extraction cycle {}", ship.symbol);

            if is_siphon {
                ship.siphon(&self.context.api).await?;
            } else {
                ship.extract(&self.context.api).await?;
            }
            ship.notify().await;
            self.eject_blacklist(ship).await?;
        }

        Ok(())
    }

    /// Get the current waypoint for a ship
    fn get_current_waypoint(&self, ship: &ship::MyShip) -> models::Waypoint {
        self.context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .unwrap()
            .get(&ship.nav.waypoint_symbol)
            .unwrap()
            .clone()
    }

    /// Find suitable waypoints for mining or siphoning
    async fn find_suitable_waypoints(
        &self,
        system_symbol: String,
        is_siphon: bool,
    ) -> Vec<(models::Waypoint, (String, i32))> {
        let filter_fn = if is_siphon {
            models::Waypoint::is_sipherable
        } else {
            models::Waypoint::is_minable
        };

        futures::stream::iter(self.get_best_waypoints(system_symbol, filter_fn))
            .filter(|wp| {
                let places = self.mining_places.clone();
                let wp = wp.clone();
                async move {
                    places.get_count(&wp.0).await
                        < CONFIG.mining.max_miners_per_waypoint.try_into().unwrap()
                }
            })
            .collect::<Vec<_>>()
            .await
    }

    async fn eject_blacklist(&self, ship: &mut ship::MyShip) -> Result<(), anyhow::Error> {
        let cargo = ship.cargo.inventory.clone();
        for item in cargo.iter() {
            if CONFIG.mining.blacklist.contains(&item.0) {
                ship.jettison(&self.context.api, *item.0, *item.1).await?;
            }
        }
        Ok(())
    }

    fn get_best_waypoints(
        &self,
        system_symbol: String,
        filter: fn(&models::Waypoint) -> bool,
    ) -> Vec<(models::Waypoint, (String, i32))> {
        let waypoints = {
            let erg = self.context.all_waypoints.try_get(&system_symbol);

            let waypoints = if erg.is_locked() {
                log::warn!("Failed to get system: {} waiting", system_symbol);
                let erg = self.context.all_waypoints.get(&system_symbol);
                log::warn!("Got system: {} waiting", system_symbol);
                erg
            } else {
                erg.try_unwrap()
            };

            let waypoints = waypoints.unwrap();

            waypoints.clone()
        };

        let points = waypoints
            .iter()
            .map(|w| w.1.clone())
            .filter(|w| filter(w))
            .collect::<Vec<_>>();

        let markets = waypoints
            .iter()
            .map(|w| w.1.clone())
            .filter(|w| w.is_marketplace())
            .collect::<Vec<_>>();

        let mut d_points: Vec<(models::Waypoint, (String, i32))> = points
            .iter()
            .map(|wp| {
                let dis = markets
                    .iter()
                    .map(|market| {
                        let distance = self.distance_squared(wp, market);
                        (market.symbol.clone(), distance)
                    })
                    .min_by(|a, b| a.1.cmp(&b.1));

                (wp.clone(), dis.unwrap())
            })
            .collect::<Vec<_>>();

        d_points.sort_by(|a, b| a.1 .1.cmp(&b.1 .1));

        d_points
    }

    fn distance_squared(&self, a: &models::Waypoint, b: &models::Waypoint) -> i32 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        dx * dx + dy * dy
    }
}
