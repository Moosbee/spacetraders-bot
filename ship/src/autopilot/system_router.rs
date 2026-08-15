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
                "System Route Cache miss"
            );
            let route = self
                .router
                .find_route_system(start_symbol, end_symbol, &config);
            debug!(route = ?route, "Route calculated");
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
            debug!(
                start_symbol = ?start_symbol,
                end_symbol = ?end_symbol,
                system_symbol = ?self.system_symbol,
                "Not in the same system"
            );
            return None;
        }

        let start_waypoint: database::Waypoint = self.waypoints.get(start_symbol)?.clone();
        let end_waypoint: database::Waypoint = self.waypoints.get(end_symbol)?.clone();

        let visited = self.generate_tree(start_waypoint, Some(end_waypoint), config);

        super::utils::get_route(visited, start_symbol.to_string(), end_symbol.to_string())
    }

    pub fn generate_tree(
        &self,
        start_waypoint: database::Waypoint,
        end_waypoint: Option<database::Waypoint>,
        config: &SystemRouterConfig,
    ) -> HashMap<String, super::SimpleConnection> {
        let mut unvisited = self.waypoints.clone();
        let mut visited = HashMap::new();
        // may use radix-heap
        let mut to_visit = PriorityQueue::new();

        to_visit.push(
            super::SimpleConnection {
                start_symbol: String::new(),
                end_symbol: start_waypoint.symbol.to_string(),
                distance: 0.0,
                cost: 0.0,
                connection_type: super::connection::ConnectionType::Navigate {
                    nav_mode: models::ShipNavFlightMode::Drift,
                },
                re_cost: 0.0,
                end_is_marketplace: start_waypoint.is_marketplace(),
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

        while let Some((current_route, _)) = to_visit.pop()
            && !unvisited.is_empty()
        {
            if self.process_current_node(
                &current_route,
                &mut to_visit,
                &mut visited,
                &mut unvisited,
                &end_waypoint,
                if first {
                    &start_range_mode
                } else if config.only_markets && !current_route.end_is_marketplace {
                    &[]
                } else {
                    &nav_modes
                },
            ) {
                break;
            }
            first = false;
        }

        visited
    }

    fn process_current_node(
        &self,
        current_route: &super::SimpleConnection,
        to_visit: &mut PriorityQueue<super::SimpleConnection, std::cmp::Reverse<i64>>,
        visited: &mut HashMap<String, super::SimpleConnection>,
        unvisited: &mut HashMap<String, database::Waypoint>,
        end_waypoint: &Option<database::Waypoint>,
        nav_modes: &[super::nav_mode::Mode],
    ) -> bool {
        let current = unvisited.remove(&current_route.end_symbol);

        if current.is_none() {
            return false;
        }

        visited.insert(current_route.end_symbol.clone(), current_route.clone());

        let current = current.unwrap();

        if let Some(end_waypoint) = end_waypoint
            && current.symbol == end_waypoint.symbol
        {
            return true;
        }

        debug!(
            current_symbol = ?current.symbol,
            from_symbol = ?current_route.start_symbol,
            cost = ?current_route.cost,
            "process_current_node"
        );

        self.explore_neighbors(
            &current,
            current_route,
            unvisited,
            to_visit,
            end_waypoint,
            nav_modes,
        );

        false
    }

    fn explore_neighbors(
        &self,
        current: &database::Waypoint,
        current_route: &super::SimpleConnection,
        unvisited: &mut HashMap<String, database::Waypoint>,
        to_visit: &mut PriorityQueue<super::SimpleConnection, std::cmp::Reverse<i64>>,
        end_waypoint: &Option<database::Waypoint>,
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

            debug!(
                current_symbol = ?current.symbol,
                mode = ?mode,
                nearby_count=nearby.len(),
                "explore_neighbors"
            );

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
        _end_waypoint: &Option<database::Waypoint>,
    ) -> super::SimpleConnection {
        let distance = distance_between_waypoints(current.into(), next.into());
        // let heuristic_cost =
        //     (distance_between_waypoints(current.into(), end_waypoint.into()) * 0.4) + 1.0;
        let heuristic_cost = 0.0;
        let cost = current_route.cost + (distance * mode.cost_multiplier) + 1.0;
        let real_cost = cost + heuristic_cost;

        super::SimpleConnection {
            start_symbol: current.symbol.clone(),
            end_symbol: next.symbol.clone(),
            distance,
            cost,
            connection_type: super::connection::ConnectionType::Navigate {
                nav_mode: mode.mode,
            },
            re_cost: real_cost,
            start_is_marketplace: current.is_marketplace(),
            end_is_marketplace: next.is_marketplace(),
        }
    }

    #[cfg(test)]
    pub fn get_waypoint(&self, symbol: &str) -> Option<&database::Waypoint> {
        self.waypoints.get(symbol)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use test_log::test;
    use tracing::info;

    use space_traders_client::models::ShipNavFlightMode;

    use super::super::NavMode;
    use super::super::connection::ConnectionType;
    use super::{SystemRouter, SystemRouterCache, SystemRouterConfig};

    const SYSTEM: &str = "X1-JP44";

    fn real_router() -> SystemRouter {
        let waypoints: Vec<database::Waypoint> = utils::tests::get_waypoints();
        let map: HashMap<String, database::Waypoint> = waypoints
            .into_iter()
            .map(|waypoint| (waypoint.symbol.clone(), waypoint))
            .collect();
        SystemRouter::new(SYSTEM.to_string(), map)
    }

    fn real_config() -> SystemRouterConfig {
        SystemRouterConfig {
            max_fuel: 400,
            max_cargo: 40,
            start_range: None,
            only_markets: true,
            nav_mode: NavMode::BurnAndCruiseAndDrift,
        }
    }

    #[test]
    fn finds_route_across_real_system() {
        let router = real_router();
        let route = router
            .find_route_system("X1-JP44-A1", "X1-JP44-J63", &real_config())
            .expect("a route should exist between two waypoints of X1-JP44");

        assert!(!route.is_empty());
        assert_eq!(route.first().unwrap().start_symbol, "X1-JP44-A1");
        assert_eq!(route.last().unwrap().end_symbol, "X1-JP44-J63");

        // the fastest way from A1 to J63 is via A1 -> E-Cluster -> I61 -> I60 -> J62 -> J63

        info!("{:?}", route);

        assert_eq!(route.len(), 5);
        assert_eq!(route[0].start_symbol, "X1-JP44-A1");
        assert_eq!(route[1].end_symbol, "X1-JP44-I61");
        assert_eq!(route[2].start_symbol, "X1-JP44-I61");
        assert_eq!(route[3].start_symbol, "X1-JP44-I60");
        assert_eq!(route[4].start_symbol, "X1-JP44-J62");
        assert_eq!(route[4].end_symbol, "X1-JP44-J63");

        for connection in &route {
            assert!(connection.start_symbol.starts_with(SYSTEM));
            assert!(connection.end_symbol.starts_with(SYSTEM));
        }
    }

    #[test]
    fn generates_full_tree() {
        let router = real_router();
        let start_waypoint = router.get_waypoint("X1-JP44-A1").unwrap().clone();

        let tree = router.generate_tree(start_waypoint, None, &real_config());

        info!("Tree: {:#?}", tree);

        assert!(tree.len() == 89);
    }

    #[test]
    fn generates_probe_tree() {
        let router = real_router();
        let start_waypoint = router.get_waypoint("X1-JP44-A1").unwrap().clone();

        let config = SystemRouterConfig {
            max_fuel: 0,
            max_cargo: 0,
            start_range: None,
            only_markets: true,
            nav_mode: NavMode::BurnAndCruiseAndDrift,
        };

        let tree = router.generate_tree(start_waypoint, None, &config);

        info!("Tree: {:#?}", tree);

        // all connections should be direct and alway Cruise

        for connection in tree.values() {
            if connection.start_symbol.is_empty() {
                assert_eq!(
                    connection.connection_type,
                    ConnectionType::Navigate {
                        nav_mode: ShipNavFlightMode::Drift
                    }
                );
                assert_eq!(connection.end_symbol, "X1-JP44-A1");
                continue;
            }
            assert_eq!(
                connection.connection_type,
                ConnectionType::Navigate {
                    nav_mode: ShipNavFlightMode::Cruise
                }
            );
            assert_eq!(connection.start_symbol, "X1-JP44-A1");
        }
    }

    #[test]
    fn returns_none_when_start_or_end_is_missing() {
        let router = real_router();

        assert!(
            router
                .find_route_system("X1-JP44-OTHER", "X1-JP44-C43", &real_config())
                .is_none()
        );
        assert!(
            router
                .find_route_system("X1-JP44-A1", "X1-JP44-OTHER", &real_config())
                .is_none()
        );
    }

    #[test]
    fn route_to_self_is_empty() {
        let router = real_router();
        let route = router
            .find_route_system("X1-JP44-A1", "X1-JP44-A1", &real_config())
            .expect("a route to the same waypoint should exist");

        assert!(route.is_empty());
    }

    #[test]
    fn returns_none_for_waypoint_in_another_system() {
        let router = real_router();
        assert!(
            router
                .find_route_system("X1-OTHER-A", "X1-JP44-A1", &real_config())
                .is_none()
        );
    }

    #[test]
    fn cache_returns_route_and_hits_cache_on_second_call() {
        let mut cache = SystemRouterCache::new(real_router());

        let first = cache
            .find_route_system("X1-JP44-A1", "X1-JP44-J63", real_config())
            .expect("a route should exist")
            .to_vec();
        let second = cache
            .find_route_system("X1-JP44-A1", "X1-JP44-J63", real_config())
            .expect("a cached route should exist");

        assert_eq!(first.as_slice(), second);
    }
}
