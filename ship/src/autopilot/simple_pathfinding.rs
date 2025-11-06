use std::collections::HashMap;

use priority_queue::PriorityQueue;
use space_traders_client::models;
use utils::{WaypointCan, distance_between_waypoints};

use super::{
    connection::SimpleConnection,
    nav_mode::{Mode, NavMode},
};

/// Simple Pathfinding, only navigates in one system
pub struct SimplePathfinder {
    pub range: u32,
    pub nav_mode: NavMode,
    pub system: HashMap<String, database::Waypoint>,
    pub start_range: u32,
    pub only_markets: bool,
}

impl SimplePathfinder {
    pub fn find_route_system(
        &self,
        start_symbol: &str,
        end_symbol: &str,
    ) -> crate::error::Result<Vec<SimpleConnection>> {
        let mut unvisited = self.system.clone();
        let mut visited = HashMap::new();
        let mut to_visit = PriorityQueue::new();

        let start_waypoint = self.get_waypoint(&unvisited, start_symbol)?.clone();
        let end_waypoint = self.get_waypoint(&unvisited, end_symbol)?.clone();

        to_visit.push(
            SimpleConnection {
                start_symbol: String::new(),
                end_symbol: start_symbol.to_string(),
                distance: 0.0,
                cost: 0.0,
                connection_type: super::connection::ConnectionType::Navigate {
                    nav_mode: models::ShipNavFlightMode::Drift,
                },
                re_cost: 0.0,
                end_is_marketplace: end_waypoint.is_marketplace(),
                start_is_marketplace: start_waypoint.is_marketplace(),
            },
            std::cmp::Reverse(0),
        );

        let nav_modes = self.nav_mode.get_flight_modes(self.range);
        let start_range_mode = self
            .nav_mode
            .get_flight_modes(self.start_range.max(1).min(self.range));

        let mut first = true;

        while let Some((current_route, _)) = to_visit.pop() {
            if self.process_current_node(
                &current_route,
                &mut to_visit,
                &mut visited,
                &mut unvisited,
                &end_waypoint,
                end_symbol,
                &nav_modes,
                first,
                &start_range_mode,
            )? {
                break;
            }
            first = false;
        }

        let route =
            super::utils::get_route(visited, start_symbol.to_string(), end_symbol.to_string());

        route.ok_or(crate::error::Error::General("Could not find route".into()))
    }

    fn get_waypoint<'a>(
        &self,
        waypoints: &'a HashMap<String, database::Waypoint>,
        symbol: &str,
    ) -> crate::error::Result<&'a database::Waypoint> {
        waypoints.get(symbol).ok_or_else(|| {
            crate::error::Error::General(format!(
                "Could not find waypoint: {} {}",
                symbol,
                waypoints
                    .iter()
                    .next()
                    .map(|f| f.0)
                    .unwrap_or(&"empty".to_string())
            ))
        })
    }

    fn process_current_node(
        &self,
        current_route: &SimpleConnection,
        to_visit: &mut PriorityQueue<SimpleConnection, std::cmp::Reverse<i64>>,
        visited: &mut HashMap<String, SimpleConnection>,
        unvisited: &mut HashMap<String, database::Waypoint>,
        end_waypoint: &database::Waypoint,
        end_symbol: &str,
        nav_modes: &Vec<Mode>,
        first: bool,
        start_range: &Vec<Mode>,
    ) -> crate::error::Result<bool> {
        *to_visit = to_visit
            .clone()
            .into_iter()
            .filter(|(c, _)| c.end_symbol != current_route.end_symbol)
            .collect();

        visited.insert(current_route.end_symbol.clone(), current_route.clone());

        let current = unvisited
            .remove(&current_route.end_symbol)
            .ok_or_else(|| crate::error::Error::General("Could not remove from queue".into()))?;

        if current.symbol == end_symbol {
            return Ok(true);
        }

        if !self.only_markets || current.is_marketplace() || first {
            let modes = if first && !current.is_marketplace() {
                start_range
            } else {
                nav_modes
            };
            self.explore_neighbors(
                &current,
                current_route,
                unvisited,
                to_visit,
                end_waypoint,
                modes,
            );
        }

        Ok(false)
    }

    fn explore_neighbors(
        &self,
        current: &database::Waypoint,
        current_route: &SimpleConnection,
        unvisited: &HashMap<String, database::Waypoint>,
        to_visit: &mut PriorityQueue<SimpleConnection, std::cmp::Reverse<i64>>,
        end_waypoint: &database::Waypoint,
        nav_modes: &Vec<Mode>,
    ) {
        for mode in nav_modes {
            let nearby =
                super::utils::get_nearby_waypoints(unvisited, (current.x, current.y), mode.radius);

            for waypoint in nearby {
                let next_route =
                    self.calculate_next_route(current, waypoint, current_route, mode, end_waypoint);
                let cost = std::cmp::Reverse((next_route.re_cost * 1_000_000.0) as i64);
                to_visit.push_increase(next_route, cost);
            }
        }
    }

    fn calculate_next_route(
        &self,
        current: &database::Waypoint,
        next: &database::Waypoint,
        current_route: &SimpleConnection,
        mode: &Mode,
        end_waypoint: &database::Waypoint,
    ) -> SimpleConnection {
        let distance = distance_between_waypoints(current.into(), next.into());
        // let heuristic_cost =
        //     (distance_between_waypoints(current.into(), end_waypoint.into()) * 0.4) + 1.0;
        let heuristic_cost = 0.0;
        let cost = current_route.cost + (distance * mode.cost_multiplier) + 1.0;

        SimpleConnection {
            start_symbol: current.symbol.clone(),
            end_symbol: next.symbol.clone(),
            distance,
            cost,
            connection_type: super::connection::ConnectionType::Navigate {
                nav_mode: mode.mode,
            },
            re_cost: cost + heuristic_cost,
            start_is_marketplace: current.is_marketplace(),
            end_is_marketplace: next.is_marketplace(),
        }
    }
}
