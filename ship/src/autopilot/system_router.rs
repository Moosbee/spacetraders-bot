use std::collections::HashMap;

use priority_queue::PriorityQueue;
use space_traders_client::models;
use tracing::debug;
use utils::{WaypointCan, distance_between_waypoints};

#[derive(Debug, Clone)]
pub struct SystemRouterCache {
    router: SystemRouter,
    cache: HashMap<(String, String, SystemRouterConfig), Vec<super::SimpleConnection>>,
}

impl SystemRouterCache {
    pub fn new(router: SystemRouter) -> Self {
        Self {
            router,
            cache: HashMap::new(),
        }
    }

    pub fn find_route_system(
        &mut self,
        start_symbol: &str,
        end_symbol: &str,
        config: SystemRouterConfig,
    ) -> Option<&[super::SimpleConnection]> {
        if !self
            .cache
            .contains_key(&(start_symbol.to_string(), end_symbol.to_string(), config))
        {
            debug!(
                key = ?(start_symbol.to_string(), end_symbol.to_string(), config),
                "Jump Route Cache miss"
            );
            let route = self
                .router
                .find_route_system(start_symbol, end_symbol, &config);
            let route = route?;
            self.cache.insert(
                (start_symbol.to_string(), end_symbol.to_string(), config),
                route,
            );
        }

        Some(self.cache
            .get(&(start_symbol.to_string(), end_symbol.to_string(), config))
            .unwrap_or_else(|| {
                panic!(
                    "We should have imported the route in the cache(find_cached_route) key: {:?}",
                    (start_symbol.to_string(), end_symbol.to_string())
                )
            }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemRouterConfig {
    pub max_fuel: u32,
    pub max_cargo: u32,
    pub start_range: Option<u32>,
    pub only_markets: bool,
    pub nav_mode: super::NavMode,
}

impl From<super::ShipNavStats> for SystemRouterConfig {
    fn from(value: super::ShipNavStats) -> Self {
        Self {
            max_fuel: value.max_fuel,
            max_cargo: value.max_cargo,
            start_range: value.start_range,
            only_markets: value.only_markets,
            nav_mode: value.nav_mode,
        }
    }
}
impl From<&super::ShipNavStats> for SystemRouterConfig {
    fn from(value: &super::ShipNavStats) -> Self {
        Self {
            max_fuel: value.max_fuel,
            max_cargo: value.max_cargo,
            start_range: value.start_range,
            only_markets: value.only_markets,
            nav_mode: value.nav_mode,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemRouter {
    pub system_symbol: String,
    pub waypoints: HashMap<String, database::Waypoint>,
}

impl SystemRouter {
    pub fn new(system_symbol: String, waypoints: HashMap<String, database::Waypoint>) -> Self {
        Self {
            system_symbol,
            waypoints,
        }
    }

    pub fn find_route_system(
        &self,
        start_symbol: &str,
        end_symbol: &str,
        config: &SystemRouterConfig,
    ) -> Option<Vec<super::SimpleConnection>> {
        if !start_symbol.starts_with(&self.system_symbol)
            || !end_symbol.starts_with(&self.system_symbol)
        {
            return None;
        }
        let mut unvisited = self.waypoints.clone();
        let mut visited = HashMap::new();
        // may use radix-heap
        let mut to_visit = PriorityQueue::new();

        let start_waypoint = unvisited.get(start_symbol)?.clone();
        let end_waypoint = unvisited.get(end_symbol)?.clone();

        to_visit.push(
            super::SimpleConnection {
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

        let nav_modes = config.nav_mode.get_flight_modes(config.max_fuel);
        let start_range_mode = config.nav_mode.get_flight_modes(
            config
                .start_range
                .unwrap_or(config.max_fuel)
                .max(1)
                .min(config.max_fuel),
        );

        let mut first = !start_waypoint.is_marketplace();

        while let Some((current_route, _)) = to_visit.pop() {
            if self.process_current_node(
                &current_route,
                &mut to_visit,
                &mut visited,
                &mut unvisited,
                &end_waypoint,
                if first { &nav_modes } else { &start_range_mode },
            )? {
                break;
            }
            first = false;
        }

        super::utils::get_route(visited, start_symbol.to_string(), end_symbol.to_string())
    }

    fn process_current_node(
        &self,
        current_route: &super::SimpleConnection,
        to_visit: &mut PriorityQueue<super::SimpleConnection, std::cmp::Reverse<i64>>,
        visited: &mut HashMap<String, super::SimpleConnection>,
        unvisited: &mut HashMap<String, database::Waypoint>,
        end_waypoint: &database::Waypoint,
        nav_modes: &[super::nav_mode::Mode],
    ) -> Option<bool> {
        visited.insert(current_route.end_symbol.clone(), current_route.clone());

        let current = unvisited.remove(&current_route.end_symbol)?;

        if current.symbol == end_waypoint.symbol {
            return Some(true);
        }

        self.explore_neighbors(
            &current,
            current_route,
            unvisited,
            to_visit,
            end_waypoint,
            nav_modes,
        );

        Some(false)
    }

    fn explore_neighbors(
        &self,
        current: &database::Waypoint,
        current_route: &super::SimpleConnection,
        unvisited: &mut HashMap<String, database::Waypoint>,
        to_visit: &mut PriorityQueue<super::SimpleConnection, std::cmp::Reverse<i64>>,
        end_waypoint: &database::Waypoint,
        nav_modes: &[super::nav_mode::Mode],
    ) {
        let mut last_radius = -1.0;
        for mode in nav_modes {
            let nearby = super::utils::get_nearby_waypoints_donut(
                unvisited,
                (current.x, current.y),
                last_radius,
                mode.radius,
            );
            last_radius = mode.radius;

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
        current_route: &super::SimpleConnection,
        mode: &super::nav_mode::Mode,
        _end_waypoint: &database::Waypoint,
    ) -> super::SimpleConnection {
        let distance = distance_between_waypoints(current.into(), next.into());
        // let heuristic_cost =
        //     (distance_between_waypoints(current.into(), end_waypoint.into()) * 0.4) + 1.0;
        let heuristic_cost = 0.0;
        let cost = current_route.cost + (distance * mode.cost_multiplier) + 1.0;

        super::SimpleConnection {
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
