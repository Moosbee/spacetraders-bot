use std::{
    collections::{HashMap, HashSet},
    string,
    sync::Arc,
    time::Duration,
};

use anyhow::Ok;
use dashmap::DashMap;
use futures::FutureExt;
use log::{debug, error, info};
use space_traders_client::models::{self, waypoint};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{
    config::CONFIG,
    ship::{self, Role},
    sql::TransactionReason,
    types::{safely_get_map, safely_get_mut_map, WaypointCan},
    workers::mining::m_types::MiningShipAssignment,
};

#[derive(Debug, Clone)]
pub struct MiningFleet {
    context: crate::workers::types::ConductorContext,
    cancellation_token: CancellationToken,
    mining_places: Arc<DashMap<String, HashSet<String>>>,
}

impl MiningFleet {
    #[allow(dead_code)]
    pub fn new_box(_context: crate::workers::types::ConductorContext) -> Box<Self> {
        Box::new(MiningFleet {
            context: _context,
            cancellation_token: CancellationToken::new(),
            mining_places: Arc::new(DashMap::new()),
        })
    }

    fn assign_to(&self, ship_symbol: &String, waypoint: &waypoint::Waypoint) -> bool {
        let symbol = waypoint.symbol.clone();

        let system: Option<dashmap::mapref::one::RefMut<'_, String, HashSet<String>>> =
            safely_get_mut_map(&self.mining_places, &symbol);

        if system.is_none() {
            debug!("No mining place for {}", symbol);
            let had = self.mining_places.insert(symbol.clone(), HashSet::new());
            if had.is_some() {
                log::warn!("Dafug Dashmap race {}", symbol); // This should not happen
                for ship_i in had.unwrap().iter() {
                    self.assign_to(ship_i, &waypoint);
                }
            }
            return self.assign_to(ship_symbol, waypoint);
        }

        let mut system = system.unwrap();

        if system.contains(ship_symbol) {
            return true;
        }

        if system.len() >= CONFIG.mining.max_miners_per_waypoint.try_into().unwrap() {
            return false;
        }

        system.insert(ship_symbol.clone());

        true
    }

    fn get_count(&self, waypoint: &waypoint::Waypoint) -> usize {
        let system = self.mining_places.try_get(&waypoint.system_symbol);

        let erg = if system.is_locked() {
            log::warn!(
                "Failed to get mining place: {} waiting",
                waypoint.system_symbol
            );
            let system = self.mining_places.get(&waypoint.system_symbol);
            log::warn!("Got mining place: {} waiting", waypoint.system_symbol);
            system
        } else {
            system.try_unwrap()
        };
        erg.map(|s| s.len()).unwrap_or(0)
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
                                    "We got Error: {} {:?} {:?} {:?}",
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
        let mut ship = self
            .context
            .ship_manager
            .get_ship_mut(&ship_symbol)
            .unwrap();

        if let Role::Mining(assignment) = ship.role {
            match assignment {
                MiningShipAssignment::Extractor => {
                    self.run_extractor_ship_worker(&mut ship).await?
                }
                MiningShipAssignment::Transporter => {
                    self.run_transporter_ship_worker(&mut ship).await?
                }
                MiningShipAssignment::Siphoner => self.run_siphoned_ship_worker(&mut ship).await?,
                MiningShipAssignment::Surveyor => self.run_surveyor_ship_worker(&mut ship).await?,
                MiningShipAssignment::Idle => {}
                MiningShipAssignment::Useless => {}
            }
        }

        Ok(())
    }

    async fn run_transporter_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        Ok(())
    }
    async fn run_extractor_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
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
            self.assign_to(&ship.symbol, &current_waypoint)
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
            let waypoints = self
                .get_best_waypoints(ship.nav.system_symbol.clone(), |w| w.is_minable())
                .into_iter()
                .filter(|wp| {
                    self.get_count(&wp.0)
                        < CONFIG.mining.max_miners_per_waypoint.try_into().unwrap()
                })
                .collect::<Vec<_>>();

            let waypoint = waypoints.first().unwrap().0.clone();

            worked = self.assign_to(&ship.symbol, &waypoint);
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
        waypoint: &waypoint::Waypoint,
    ) -> anyhow::Result<()> {
        for _ in 0..CONFIG.mining.max_extractions_per_miner {
            let i = tokio::select! {
                  _ = self.cancellation_token.cancelled() => {0},
                _=ship.wait_for_cooldown() => {1},
            };

            if i == 0 {
                break;
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
            self.assign_to(&ship.symbol, &current_waypoint)
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
            let waypoints = self
                .get_best_waypoints(ship.nav.system_symbol.clone(), |w| w.is_sipherable())
                .into_iter()
                .filter(|wp| {
                    self.get_count(&wp.0)
                        < CONFIG.mining.max_miners_per_waypoint.try_into().unwrap()
                })
                .collect::<Vec<_>>();

            let waypoint = waypoints.first().unwrap().0.clone();

            worked = self.assign_to(&ship.symbol, &waypoint);
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
        waypoint: &waypoint::Waypoint,
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

    async fn run_surveyor_ship_worker(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        Ok(())
    }

    fn with_cancel(&self, cancellation_token: CancellationToken) -> MiningFleet {
        MiningFleet {
            cancellation_token: cancellation_token,
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
            let mut ship = self.context.ship_manager.get_ship_mut(ship_name).unwrap();
            let ship_capabilities = self.analyze_ship_capabilities(&mut ship);

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
    fn analyze_ship_capabilities(&self, ship: &mut ship::MyShip) -> ShipCapabilities {
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
