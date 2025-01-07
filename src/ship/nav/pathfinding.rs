use super::{
    nav_models::{NavMode, RouteConnection},
    utils::distance_between_waypoints,
};
use crate::error::Result;
use crate::ship::ship_models::MyShip;
use crate::types::WaypointCan;
use priority_queue::PriorityQueue;
use space_traders_client::models;
use std::collections::HashMap;

impl MyShip {
    pub fn find_route(
        &mut self,
        waypoints: &HashMap<String, models::Waypoint>,
        start_symbol: String,
        end_symbol: String,
        nav_mode: &NavMode,
        only_markets: bool,
        start_range: i32,
    ) -> Result<Vec<RouteConnection>> {
        let mut cache = self.nav.cache.clone();
        let erg = self.find_route_cached(
            waypoints,
            start_symbol,
            end_symbol,
            nav_mode,
            only_markets,
            start_range,
            &mut cache,
        );

        self.nav.cache = cache;

        erg
    }

    pub fn find_route_cached(
        &self,
        waypoints: &HashMap<String, models::Waypoint>,
        start_symbol: String,
        end_symbol: String,
        nav_mode: &NavMode,
        only_markets: bool,
        start_range: i32,
        cache: &mut super::nav_models::Cache,
    ) -> Result<Vec<RouteConnection>> {
        if let Some(route) = cache.get(
            start_symbol.clone(),
            end_symbol.clone(),
            nav_mode,
            only_markets,
            self.fuel.capacity,
            start_range,
        ) {
            return Ok(route);
        }

        let mut unvisited = waypoints.clone();
        let mut visited = HashMap::new();
        let mut to_visit = PriorityQueue::new();

        let start = self.get_waypoint(&unvisited, &start_symbol)?;
        let end_waypoint = self.get_waypoint(&unvisited, &end_symbol)?.clone();

        to_visit.push(
            RouteConnection {
                start_symbol: String::new(),
                end_symbol: start.symbol.clone(),
                distance: 0.0,
                cost: 0.0,
                flight_mode: models::ShipNavFlightMode::Drift,
                re_cost: 0.0,
            },
            std::cmp::Reverse(0),
        );

        let nav_modes = nav_mode.get_flight_modes(self.fuel.capacity);
        let start_range_mode = nav_mode.get_flight_modes(start_range);

        let mut first = true;

        while let Some((current_route, _)) = to_visit.pop() {
            if self.process_current_node(
                &current_route,
                &mut to_visit,
                &mut visited,
                &mut unvisited,
                &end_waypoint,
                &end_symbol,
                only_markets,
                &nav_modes,
                first,
                &start_range_mode,
            )? {
                break;
            }
            first = false;
        }

        let route = super::utils::get_route(visited, start_symbol.clone(), end_symbol.clone());

        if let Ok(route) = &route {
            cache.put(
                start_symbol.clone(),
                end_symbol.clone(),
                nav_mode,
                only_markets,
                self.fuel.capacity,
                start_range,
                route.clone(),
            );
        }

        route
    }

    fn get_waypoint<'a>(
        &self,
        waypoints: &'a HashMap<String, models::Waypoint>,
        symbol: &str,
    ) -> Result<&'a models::Waypoint> {
        waypoints.get(symbol).ok_or_else(|| {
            crate::error::Error::General(format!("Could not find waypoint: {}", symbol))
        })
    }

    fn process_current_node(
        &self,
        current_route: &RouteConnection,
        to_visit: &mut PriorityQueue<RouteConnection, std::cmp::Reverse<i64>>,
        visited: &mut HashMap<String, RouteConnection>,
        unvisited: &mut HashMap<String, models::Waypoint>,
        end_waypoint: &models::Waypoint,
        end_symbol: &str,
        only_markets: bool,
        nav_modes: &Vec<super::nav_models::Mode>,
        first: bool,
        start_range: &Vec<super::nav_models::Mode>,
    ) -> Result<bool> {
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

        if !only_markets || current.is_marketplace() || first {
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
        current: &models::Waypoint,
        current_route: &RouteConnection,
        unvisited: &HashMap<String, models::Waypoint>,
        to_visit: &mut PriorityQueue<RouteConnection, std::cmp::Reverse<i64>>,
        end_waypoint: &models::Waypoint,
        nav_modes: &Vec<super::nav_models::Mode>,
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
        current: &models::Waypoint,
        next: &models::Waypoint,
        current_route: &RouteConnection,
        mode: &super::nav_models::Mode,
        end_waypoint: &models::Waypoint,
    ) -> RouteConnection {
        let distance = distance_between_waypoints((current.x, current.y), (next.x, next.y));
        let heuristic_cost =
            distance_between_waypoints((next.x, next.y), (end_waypoint.x, end_waypoint.y)) * 0.4;
        let cost = current_route.cost + (distance * mode.cost_multiplier) + 1.0;

        RouteConnection {
            start_symbol: current.symbol.clone(),
            end_symbol: next.symbol.clone(),
            distance,
            cost,
            flight_mode: mode.mode,
            re_cost: cost + heuristic_cost,
        }
    }
}
