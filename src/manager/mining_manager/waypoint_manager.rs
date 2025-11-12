use space_traders_client::models;
use utils::WaypointCan;

use crate::utils::ConductorContext;

use super::{
    mining_places::{AssignLevel, MiningPlaces},
    place_finder::{self, PlaceFinder},
    ActionType,
};

use crate::error::Result;

#[derive(Debug)]
pub struct WaypointManager {
    places: MiningPlaces,
    finder: PlaceFinder,
    context: ConductorContext,
}

impl WaypointManager {
    pub fn new(context: ConductorContext, max_miners: u32) -> Self {
        Self {
            places: MiningPlaces::new(max_miners),
            finder: PlaceFinder::new(context.clone()),
            context,
        }
    }

    // pub fn iter(&self) -> impl Iterator<Item = (&String, &WaypointInfo)> {
    //     self.places.iter()
    // }

    pub async fn assign_waypoint_syphon(
        &mut self,
        ship_clone: ship::MyShip,
        is_syphon: bool,
    ) -> Result<String> {
        // let action: ActionType = ActionType::get_action(&ship_clone).ok_or("Invalid ship role")?;
        let action: ActionType = if is_syphon {
            ActionType::Siphon
        } else {
            ActionType::Extract
        };

        self.assign_waypoint(&ship_clone, action).await
    }
    pub async fn assign_waypoint(
        &mut self,
        ship: &ship::MyShip,
        action: ActionType,
    ) -> Result<String> {
        let (ignore_engineered_asteroids, unstable_since_timeout, stop_all_unstable) = {
            let config = self.context.config.read().await;
            (
                config.ignore_engineered_asteroids,
                config.unstable_since_timeout,
                config.stop_all_unstable,
            )
        };

        if let Some((waypoint_symbol, _)) = self.places.get_ship(&ship.symbol) {
            let waypoint =
                database::Waypoint::get_by_symbol(&self.context.database_pool, &waypoint_symbol)
                    .await?;

            if waypoint
                .map(|waypoint| {
                    waypoint.is_minable()
                        && (waypoint.waypoint_type != models::WaypointType::EngineeredAsteroid
                            || ignore_engineered_asteroids)
                        && waypoint
                            .unstable_since
                            .map(|last| {
                                (last + chrono::Duration::seconds(unstable_since_timeout)
                                    < chrono::Utc::now())
                                    || !stop_all_unstable
                            })
                            .unwrap_or(true)
                })
                .unwrap_or(false)
                && self.places.try_assign_on_way(
                    &ship.symbol,
                    &waypoint_symbol,
                    action == ActionType::Siphon,
                ) != 0
            {
                return Ok(waypoint_symbol.to_string());
            }
        }

        let waypoint_symbol = ship.nav.waypoint_symbol.clone();

        let waypoint =
            database::Waypoint::get_by_symbol(&self.context.database_pool, &waypoint_symbol)
                .await?;
        if waypoint
            .map(|waypoint| {
                waypoint.is_minable()
                    && (waypoint.waypoint_type != models::WaypointType::EngineeredAsteroid
                        || ignore_engineered_asteroids)
                    && waypoint
                        .unstable_since
                        .map(|last| {
                            (last + chrono::Duration::seconds(unstable_since_timeout)
                                < chrono::Utc::now())
                                || !stop_all_unstable
                        })
                        .unwrap_or(true)
            })
            .unwrap_or(false)
            && self.places.try_assign_on_way(
                &ship.symbol,
                &waypoint_symbol,
                action == ActionType::Siphon,
            ) != 0
        {
            return Ok(waypoint_symbol.to_string());
        }

        let counts = self.places.get_max_miners_per_waypoint()
            + (((action == ActionType::Siphon) as u32) * 10000);

        let waypoints: Vec<place_finder::FoundWaypointInfo> = self
            .finder
            .find(
                ship.clone(),
                |waypoint| match action {
                    ActionType::Extract => {
                        waypoint.is_minable()
                            && (waypoint.waypoint_type != models::WaypointType::EngineeredAsteroid
                                || ignore_engineered_asteroids)
                            && waypoint
                                .unstable_since
                                .map(|last| {
                                    (last + chrono::Duration::seconds(unstable_since_timeout)
                                        < chrono::Utc::now())
                                        || !stop_all_unstable
                                })
                                .unwrap_or(true)
                    }
                    ActionType::Siphon => waypoint.is_sipherable(),
                },
                &self.places,
                counts as usize,
            )
            .await?;

        self.assign_to_available_waypoint(ship, waypoints, action)
    }

    fn assign_to_available_waypoint(
        &mut self,
        ship: &ship::MyShip,
        waypoints: Vec<place_finder::FoundWaypointInfo>,
        action: ActionType,
    ) -> Result<String> {
        tracing::debug!(ship_symbol = %ship.symbol, waypoints_count = waypoints.len(), action = ?action, "Assigning to available waypoint");
        for waypoint in waypoints {
            if self.places.try_assign_on_way(
                &ship.symbol,
                &waypoint.waypoint.symbol,
                action == ActionType::Siphon,
            ) != 0
            {
                return Ok(waypoint.waypoint.symbol.clone());
            }
        }
        Err("No suitable waypoints found".into())
    }

    pub async fn notify_waypoint(
        &mut self,
        ship_clone: ship::MyShip,
        action: ActionType,
    ) -> std::result::Result<String, crate::error::Error> {
        let waypoint_symbol = ship_clone.nav.waypoint_symbol.clone();

        let wp = self.places.try_assign_active(
            &ship_clone.symbol,
            &waypoint_symbol,
            action == ActionType::Siphon,
        );

        if wp {
            Ok(waypoint_symbol)
        } else {
            Err("Could not activate craft".into())
        }
    }

    pub async fn unassign_waypoint(
        &mut self,
        ship_clone: ship::MyShip,
    ) -> std::result::Result<String, crate::error::Error> {
        let waypoint_symbol = ship_clone.nav.waypoint_symbol.clone();

        let wp = self
            .places
            .try_assign_inactive(&ship_clone.symbol, &waypoint_symbol);

        if wp {
            Ok(waypoint_symbol)
        } else {
            Err("Could not deactivate craft".into())
        }
    }

    pub async fn unassign_waypoint_complete(
        &mut self,
        ship_clone: ship::MyShip,
    ) -> std::result::Result<String, crate::error::Error> {
        let waypoint_symbol = ship_clone.nav.waypoint_symbol.clone();

        let wp = self
            .places
            .try_unassign(&ship_clone.symbol, &waypoint_symbol);

        if wp {
            Ok(waypoint_symbol)
        } else {
            Err("Could not deactivate craft".into())
        }
    }

    pub fn up_date(&mut self, waypoint: &str) {
        self.places.up_date(waypoint);
    }

    pub fn get_all_places(&self) -> Vec<(String, super::mining_places::WaypointInfo)> {
        self.places
            .iter()
            .map(|wp| (wp.0.clone(), wp.1.clone()))
            .collect::<Vec<_>>()
    }

    pub fn calculate_waypoint_urgencys(
        &self,
        the_ships: &std::collections::HashMap<String, ship::MyShip>,
    ) -> Vec<(String, i64)> {
        let mut erg = self
            .places
            .iter()
            .map(|wp| Self::calculate_waypoint_urgency(wp.1, the_ships))
            .collect::<Vec<_>>();

        erg.sort_by(|a, b| b.1.cmp(&a.1));

        erg
    }

    // where do I put you?
    fn calculate_waypoint_urgency(
        wp: &super::mining_places::WaypointInfo,
        ships: &std::collections::HashMap<String, ship::MyShip>,
    ) -> (String, i64) {
        let (units_sum, capacity_sum) = wp
            .ship_iter()
            .filter(|s| s.1 == &AssignLevel::Active)
            .filter_map(|s| ships.get(s.0))
            .filter(|sh| sh.nav.waypoint_symbol == wp.waypoint_symbol && !sh.nav.is_in_transit())
            .map(|sh| (sh.cargo.units, sh.cargo.capacity))
            .fold((0, 0), |(units_sum, capacity_sum), (units, capacity)| {
                (units_sum + units, capacity_sum + capacity)
            });

        let (_units_sum_on_way, _capacity_sum_on_way, earliest_arrival) = ships
            .iter()
            .map(|s| s.1)
            .filter(|s| {
                matches!(
                    s.status.status,
                    ship::AssignmentStatus::Mining {
                        assignment: ship::status::MiningShipAssignment::Extractor { .. }
                    }
                )
            })
            .filter(|sh| {
                sh.nav.auto_pilot.as_ref().map(|a| &a.destination_symbol)
                    == Some(&wp.waypoint_symbol)
            })
            .map(|sh| {
                (
                    sh.cargo.units,
                    sh.cargo.capacity,
                    sh.nav.auto_pilot.as_ref().map(|a| a.arrival),
                )
            })
            .fold(
                (0, 0, chrono::DateTime::<chrono::Utc>::MAX_UTC),
                |(units_sum, capacity_sum, min_arrival), (units, capacity, arrival_time)| {
                    (
                        units_sum + units,
                        capacity_sum + capacity,
                        arrival_time
                            .unwrap_or(chrono::DateTime::<chrono::Utc>::MIN_UTC)
                            .min(min_arrival),
                    )
                },
            );

        let cargo_ratio = (units_sum as f32 / capacity_sum as f32) * 100.0;
        let cargo_ratio = if cargo_ratio.is_nan() {
            0.0
        } else {
            cargo_ratio
        };

        let since_last = wp.get_last_updated() - chrono::Utc::now();

        let to_next = (chrono::Utc::now() - earliest_arrival).max(chrono::Duration::seconds(0));

        let urgency = (since_last.num_seconds() + to_next.num_seconds()) * cargo_ratio as i64;

        (wp.waypoint_symbol.clone(), urgency)
    }
}
