use std::{sync::Arc, time::Duration};

use anyhow::Ok;
use chrono::Utc;
use futures::{FutureExt, StreamExt};
use log::{debug, error, info};
use space_traders_client::models::{self, waypoint};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{
    config::CONFIG,
    ship::{self, Role},
    sql::TransactionReason,
    types::{safely_get_map, WaypointCan},
    workers::mining::m_types::MiningShipAssignment,
};

use super::mining_manager::{MiningManager, WaypointInfo};

#[derive(Debug, Clone)]
pub struct MiningFleet {
    context: crate::workers::types::ConductorContext,
    cancellation_token: CancellationToken,
    mining_places: Arc<MiningManager>,
}

impl MiningFleet {
    #[allow(dead_code)]
    pub fn new_box(_context: crate::workers::types::ConductorContext) -> Box<Self> {
        Box::new(MiningFleet {
            context: _context,
            cancellation_token: CancellationToken::new(),
            mining_places: Arc::new(MiningManager::new()),
        })
    }

    async fn run_mining_worker(&self) -> anyhow::Result<()> {
        info!("Starting mining workers");

        if !CONFIG.mining.active {
            info!("mining workers not active, exiting");
            return Ok(());
        }

        tokio::select! {
        _ = self.cancellation_token.cancelled() => {
          info!("Mining cancelled");
          0},
        _ =  sleep(Duration::from_millis(CONFIG.mining.start_sleep_duration)) => {1}
        };

        let ships = self.get_mining_ships();

        self.assign_ships(&ships).await;

        let mut handles = Vec::new();

        for ship in ships {
            let child_stopper = self.cancellation_token.child_token();

            let fleet = self.with_cancel(child_stopper.clone());
            let handle = tokio::spawn(async move { fleet.run_mining_ship_worker(ship).await })
                .then(|result| async move {
                    if let Err(e) = &result {
                        error!("Lel Error: {}", e);
                    }
                    match &result {
                        Result::Ok(result) => {
                            if let Err(e) = result {
                                error!(
                                    "We got Mining Error: {} {:?} {:?} {:?}",
                                    e,
                                    e.backtrace(),
                                    e.source(),
                                    e.root_cause()
                                );
                            }
                        }
                        _ => (),
                    }
                    result
                });
            handles.push((handle, child_stopper));
        }

        for (handle, _child_stopper) in handles {
            let _ = handle.await;
        }

        info!("mining workers done");

        Ok(())
    }

    fn get_mining_ships(&self) -> Vec<String> {
        self.context
            .ship_roles
            .iter()
            .filter_map(|(symbol, role)| match role {
                ship::Role::Mining(_) => Some(symbol.clone()),
                _ => None,
            })
            .collect()
    }

    async fn run_mining_ship_worker(&self, ship_symbol: String) -> anyhow::Result<()> {
        let mut guard = self.context.ship_manager.get_mut(&ship_symbol).await;
        let ship = guard.value_mut().unwrap();

        if let Role::Mining(assignment) = ship.role {
            match assignment {
                MiningShipAssignment::Extractor => self.run_extractor_ship_worker(ship).await?,
                MiningShipAssignment::Transporter => self.run_transporter_ship_worker(ship).await?,
                MiningShipAssignment::Siphoner => self.run_siphoned_ship_worker(ship).await?,
                MiningShipAssignment::Surveyor => self.run_surveyor_ship_worker(ship).await?,
                MiningShipAssignment::Idle => {}
                MiningShipAssignment::Useless => {}
            }
        }

        Ok(())
    }

    async fn run_transporter_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(
            500 + rand::random::<u64>() % 500,
        ))
        .await;

        let waypoints = self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .unwrap()
            .clone();

        for _i in 0..CONFIG.mining.max_extractions_per_miner {
            if self.cancellation_token.is_cancelled() {
                info!("Transport cycle cancelled for {} ", ship.symbol);
                break;
            }
            let route = self.calculate_waypoint_urgencys().await;
            debug!("Routes: {:?}", route);
            let routes = route.iter().filter(|r| r.1 > 0).collect::<Vec<_>>();

            if routes.is_empty() {
                info!("No routes found for {}", ship.symbol);
                tokio::time::sleep(std::time::Duration::from_millis(
                    5000 + rand::random::<u64>() % 5000,
                ))
                .await;
                continue;
            }

            let route = routes.last().unwrap();
            debug!("Route: {:?}", route);

            ship.nav_to(
                &route.0,
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                TransactionReason::None,
            )
            .await?;
        }
        Ok(())
    }

    async fn calculate_waypoint_urgencys(&self) -> Vec<(String, u32)> {
        let waypoints = self.mining_places.get_all().await;
        let the_ships: std::collections::HashMap<String, ship::MyShip> =
            self.context.ship_manager.get_all_clone();

        let mut erg = waypoints
            .iter()
            .map(|wp| Self::calculate_waypoint_urgency(wp.1, the_ships.clone()))
            .collect::<Vec<_>>();

        erg.sort_by(|a, b| b.1.cmp(&a.1));

        erg
    }

    fn calculate_waypoint_urgency(
        wp: &WaypointInfo,
        ships: std::collections::HashMap<String, ship::MyShip>,
    ) -> (String, u32) {
        let (units_sum, capacity_sum) =
            wp.1.iter()
                .map(|s| ships.get(s).unwrap())
                .filter(|sh| sh.nav.waypoint_symbol == wp.0 && !sh.nav.is_in_transit())
                .map(|sh| (sh.cargo.units, sh.cargo.capacity))
                .fold((0, 0), |(units_sum, capacity_sum), (units, capacity)| {
                    (units_sum + units, capacity_sum + capacity)
                });

        let cargo_ratio = (units_sum as f32 / capacity_sum as f32) * 100.0;
        let cargo_ratio = if cargo_ratio.is_nan() {
            0.0
        } else {
            cargo_ratio
        };

        let since_last = wp.2 - Utc::now();

        let urgency = since_last.num_seconds() * cargo_ratio as i64;

        (wp.0.clone(), urgency as u32)
    }

    async fn run_extractor_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(
            100 + rand::random::<u64>() % 100,
        ))
        .await;

        let current_waypoint = {
            self.context
                .all_waypoints
                .get(&ship.nav.system_symbol)
                .unwrap()
                .get(&ship.nav.waypoint_symbol)
                .unwrap()
                .clone()
        };

        let mut wp = None;

        let mut worked = if current_waypoint.is_minable() {
            self.mining_places
                .assign_to(&ship.symbol, &current_waypoint)
                .await
        } else {
            false
        };

        if worked {
            wp = Some(current_waypoint);
        }

        while !worked {
            tokio::time::sleep(std::time::Duration::from_millis(
                100 + rand::random::<u64>() % 100,
            ))
            .await;

            let waypoints = futures::stream::iter(
                self.get_best_waypoints(ship.nav.system_symbol.clone(), |w| w.is_minable()),
            )
            .filter(|wp| {
                let places = self.mining_places.clone();
                let wp = wp.clone();
                async move {
                    places.get_count(&wp.0).await
                        < CONFIG.mining.max_miners_per_waypoint.try_into().unwrap()
                }
            })
            .collect::<Vec<_>>()
            .await;

            let waypoint = waypoints.first().unwrap().0.clone();

            worked = self.mining_places.assign_to(&ship.symbol, &waypoint).await;
            if worked {
                wp = Some(waypoint);
            }
        }

        if let Some(waypoint) = wp {
            let system_symbol = waypoint.system_symbol.clone();
            let waypoints = {
                safely_get_map(&self.context.all_waypoints, &system_symbol)
                    .unwrap()
                    .clone()
            };

            ship.nav_to(
                waypoint.symbol.as_str(),
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                TransactionReason::None,
            )
            .await?;

            self.run_extraction_cycle(ship, &waypoint).await?
        }

        Ok(())
    }

    async fn run_extraction_cycle(
        &self,
        ship: &mut ship::MyShip,
        _waypoint: &waypoint::Waypoint,
    ) -> anyhow::Result<()> {
        for _ in 0..CONFIG.mining.max_extractions_per_miner {
            let i = tokio::select! {
                  _ = self.cancellation_token.cancelled() => {0},
                _=ship.wait_for_cooldown() => {1},
            };

            if i == 0 {
                break;
            }

            if ship.cargo.units >= ship.cargo.capacity {
                tokio::time::sleep(std::time::Duration::from_millis(
                    1000 + rand::random::<u64>() % 1000,
                ))
                .await;
                continue;
            }

            ship.extract(&self.context.api).await?;
        }

        Ok(())
    }
    async fn run_siphoned_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(
            1000 + rand::random::<u64>() % 1000,
        ))
        .await;

        let current_waypoint = {
            self.context
                .all_waypoints
                .get(&ship.nav.system_symbol)
                .unwrap()
                .get(&ship.nav.waypoint_symbol)
                .unwrap()
                .clone()
        };

        let mut wp = None;

        let mut worked = if current_waypoint.is_minable() {
            self.mining_places
                .assign_to(&ship.symbol, &current_waypoint)
                .await
        } else {
            false
        };

        if worked {
            wp = Some(current_waypoint);
        }

        while !worked {
            tokio::time::sleep(std::time::Duration::from_millis(
                1000 + rand::random::<u64>() % 1000,
            ))
            .await;

            let waypoints = futures::stream::iter(
                self.get_best_waypoints(ship.nav.system_symbol.clone(), |w| w.is_sipherable()),
            )
            .filter(|wp| {
                let places = self.mining_places.clone();
                let wp = wp.clone();
                async move {
                    places.get_count(&wp.0).await
                        < CONFIG.mining.max_miners_per_waypoint.try_into().unwrap()
                }
            })
            .collect::<Vec<_>>()
            .await;

            let waypoint = waypoints.first().unwrap().0.clone();

            worked = self.mining_places.assign_to(&ship.symbol, &waypoint).await;
            if worked {
                wp = Some(waypoint);
            }
        }

        if let Some(waypoint) = wp {
            let system_symbol = waypoint.system_symbol.clone();
            let waypoints = {
                safely_get_map(&self.context.all_waypoints, &system_symbol)
                    .unwrap()
                    .clone()
            };

            ship.nav_to(
                waypoint.symbol.as_str(),
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                TransactionReason::None,
            )
            .await?;

            self.run_siphoning_cycle(ship, &waypoint).await?
        }

        Ok(())
    }

    async fn run_siphoning_cycle(
        &self,
        ship: &mut ship::MyShip,
        _waypoint: &waypoint::Waypoint,
    ) -> anyhow::Result<()> {
        for _ in 0..CONFIG.mining.max_extractions_per_miner {
            let i = tokio::select! {
                  _ = self.cancellation_token.cancelled() => {0},
                _=ship.wait_for_cooldown() => {1},
            };

            if i == 0 {
                break;
            }

            ship.siphon(&self.context.api).await?;
        }

        Ok(())
    }

    async fn run_surveyor_ship_worker(&self, _ship: &mut ship::MyShip) -> anyhow::Result<()> {
        Ok(())
    }

    fn with_cancel(&self, cancellation_token: CancellationToken) -> MiningFleet {
        MiningFleet {
            cancellation_token: cancellation_token,
            mining_places: Arc::clone(&self.mining_places),
            ..self.clone()
        }
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

    async fn assign_ships(&self, ships: &Vec<String>) {
        for ship_name in ships {
            let mut guard = self.context.ship_manager.get_mut(ship_name).await;
            let ship = guard.value_mut().unwrap();
            let ship_capabilities = self.analyze_ship_capabilities(ship);

            ship.role = match ship_capabilities {
                ShipCapabilities {
                    can_extract: true,
                    can_cargo: true,
                    ..
                } => Role::Mining(MiningShipAssignment::Extractor),

                ShipCapabilities {
                    can_extract: false,
                    can_siphon: true,
                    can_cargo: true,
                    ..
                } => Role::Mining(MiningShipAssignment::Siphoner),

                ShipCapabilities {
                    can_survey: true, ..
                } => Role::Mining(MiningShipAssignment::Surveyor),

                ShipCapabilities {
                    can_cargo: true, ..
                } => Role::Mining(MiningShipAssignment::Transporter),

                _ => Role::Mining(MiningShipAssignment::Useless),
            };

            ship.notify().await;
        }
    }
    fn analyze_ship_capabilities(&self, ship: &ship::MyShip) -> ShipCapabilities {
        ShipCapabilities {
            can_extract: ship.mounts.can_extract(),
            can_siphon: ship.mounts.can_siphon(),
            can_survey: ship.mounts.can_survey(),
            can_cargo: ship.cargo.capacity > 0,
        }
    }
}

#[derive(Debug, Clone)]
struct ShipCapabilities {
    can_extract: bool,
    can_siphon: bool,
    can_survey: bool,
    can_cargo: bool,
}

impl crate::workers::types::Conductor for MiningFleet {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_mining_worker().await })
    }

    fn get_name(&self) -> String {
        "MiningFleet".to_string()
    }
    fn get_cancel_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }
}
