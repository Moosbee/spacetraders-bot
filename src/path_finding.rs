use std::{cmp::Reverse, collections::HashMap};

use anyhow::{Error, Ok};
use chrono::{Duration, TimeDelta};
use priority_queue::PriorityQueue;
use space_traders_client::models::{self, Waypoint};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NavMode {
    Burn,
    Cruise,
    Drift,
    BurnAndCruise,
    CruiseAndDrift,
    BurnAndDrift,
    BurnAndCruiseAndDrift,
}
struct Mode {
    radius: f64,
    cost_multiplier: f64,
    mode: models::ShipNavFlightMode,
}
struct Modes {
    burn: Mode,
    cruise: Mode,
    drift: Mode,
}
impl NavMode {
    fn get_modes(&self, all_modes: Modes) -> Vec<Mode> {
        let mut modes = Vec::new();
        if self == &NavMode::Burn
            || self == &NavMode::BurnAndCruise
            || self == &NavMode::BurnAndDrift
            || self == &NavMode::BurnAndCruiseAndDrift
        {
            modes.push(all_modes.burn);
        }
        if self == &NavMode::Cruise
            || self == &NavMode::BurnAndCruise
            || self == &NavMode::CruiseAndDrift
            || self == &NavMode::BurnAndCruiseAndDrift
        {
            modes.push(all_modes.cruise);
        }
        if self == &NavMode::Drift
            || self == &NavMode::CruiseAndDrift
            || self == &NavMode::BurnAndDrift
            || self == &NavMode::BurnAndCruiseAndDrift
        {
            modes.push(all_modes.drift);
        }

        modes
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouteConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub flight_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub cost: f64,
    pub re_cost: f64,
}

impl PartialEq for RouteConnection {
    fn eq(&self, other: &Self) -> bool {
        self.end_symbol == other.end_symbol
            && self.start_symbol == other.start_symbol
            && self.flight_mode == other.flight_mode
    }
}
impl Eq for RouteConnection {}

impl std::hash::Hash for RouteConnection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.end_symbol.hash(state);
        self.start_symbol.hash(state);
        self.flight_mode.hash(state);
    }
}

pub fn get_route_a_star(
    waypoints: &HashMap<String, models::Waypoint>,
    start_symbol: String,
    end_symbol: String,
    max_fuel: i32,
    nav_mode: NavMode,
    only_markets: bool,
) -> Result<Vec<RouteConnection>, Error> {
    let mut unvisited: HashMap<String, models::Waypoint> = waypoints.clone();
    // .clone()
    // .iter()
    // .map(|w| (w.symbol.clone(), w.clone()))
    // .collect();
    let mut visited: HashMap<String, RouteConnection> = HashMap::new();
    let mut to_visit: PriorityQueue<RouteConnection, Reverse<i64>> = PriorityQueue::new();

    let start = match unvisited.get(&start_symbol) {
        Some(it) => it,
        None => return Err(anyhow::anyhow!("Could not find start waypoint")),
    };

    let modes = nav_mode.get_modes(Modes {
        burn: Mode {
            radius: (max_fuel as f64) / 2.0,
            cost_multiplier: 0.5,
            mode: models::ShipNavFlightMode::Burn,
        },
        cruise: Mode {
            radius: (max_fuel as f64),
            cost_multiplier: 1.0,
            mode: models::ShipNavFlightMode::Cruise,
        },
        drift: Mode {
            radius: f64::INFINITY,
            cost_multiplier: 10.0,
            mode: models::ShipNavFlightMode::Drift,
        },
    });

    to_visit.push(
        RouteConnection {
            start_symbol: "".to_string(),
            end_symbol: start.symbol.clone(),
            distance: 0.0,
            cost: 0.0,
            flight_mode: models::ShipNavFlightMode::Drift,
            re_cost: 0.0,
        },
        Reverse(0),
    );

    let end_waypoint = unvisited
        .get(&end_symbol)
        .ok_or(anyhow::anyhow!(
            "Could not find end waypoint: {}",
            end_symbol
        ))?
        .clone();

    while !to_visit.is_empty() {
        let (current_route, _) = to_visit
            .pop()
            .ok_or(anyhow::anyhow!("Could not pop from queue"))?;
        to_visit = to_visit
            .into_iter()
            .filter(|(c, _)| current_route.end_symbol != c.end_symbol)
            .collect();
        visited.insert(current_route.end_symbol.clone(), current_route.clone());
        let current = unvisited
            .remove(&current_route.end_symbol)
            .ok_or(anyhow::anyhow!("Could not remove from queue"))?;

        if current.symbol == end_symbol {
            break;
        }

        if !only_markets
            || current
                .traits
                .iter()
                .any(|t| t.symbol == models::WaypointTraitSymbol::Marketplace)
        {
            for mode in &modes {
                let next_waypoints =
                    get_waypoints_within_radius(&unvisited, (current.x, current.y), mode.radius);

                // depends on luck what waypoints are chosen first on same cost
                // next_waypoints.sort_by(|a, b| a.symbol.cmp(&b.symbol));
                // next_waypoints.sort_by(|a, b| b.symbol.cmp(&a.symbol));
                // next_waypoints.reverse();

                for waypoint in next_waypoints.iter() {
                    let distance = distance_between_waypoints(
                        (current.x, current.y),
                        (waypoint.x, waypoint.y),
                    );

                    let heuristic_cost = distance_between_waypoints(
                        (waypoint.x, waypoint.y),
                        (end_waypoint.x, end_waypoint.y),
                    ) * 0.4;

                    // let heuristic_cost = 0.0;

                    let cost = current_route.cost + (distance * mode.cost_multiplier) + 1.0;
                    let re_cost = cost + heuristic_cost;
                    let next_route = RouteConnection {
                        start_symbol: current.symbol.clone(),
                        end_symbol: waypoint.symbol.clone(),
                        distance,
                        cost,
                        flight_mode: mode.mode,
                        re_cost,
                    };

                    to_visit.push_increase(next_route, Reverse((re_cost * 1000000.0) as i64));
                }
            }
        }
    }

    get_route(visited, start_symbol, end_symbol)
}

pub fn get_route(
    waypoints: HashMap<String, RouteConnection>,
    start_symbol: String,
    end_symbol: String,
) -> Result<Vec<RouteConnection>, Error> {
    let mut route = Vec::new();
    let mut current = end_symbol;
    while current != start_symbol {
        let connection = waypoints
            .get(&current)
            .ok_or(anyhow::anyhow!("Could not find connection"))?;
        route.push(connection.clone());
        current = connection.start_symbol.clone();
    }
    route.reverse();
    Ok(route)
}

pub fn calc_route_stats(
    waypoints: &HashMap<String, models::Waypoint>,
    route: &Vec<RouteConnection>,
    engine_speed: i32,
) -> (Vec<ConnectionDetails>, f64, i32, TimeDelta) {
    let stats = route
        .iter()
        .map(|conn| {
            let start = waypoints.get(&conn.start_symbol).unwrap();
            let end = waypoints.get(&conn.end_symbol).unwrap();
            let stat = get_inter_system_travel_stats(
                engine_speed,
                conn.flight_mode,
                (start.x, start.y),
                (end.x, end.y),
            );
            ConnectionDetails {
                start: start.clone(),
                end: end.clone(),
                flight_mode: conn.flight_mode,
                distance: stat.distance,
                fuel_cost: stat.fuel_cost,
                travel_time: stat.travel_time,
            }
        })
        .collect::<Vec<_>>();

    let total_distance = stats.iter().map(|s| s.distance).sum::<f64>();
    let total_fuel_cost = stats.iter().map(|s| s.fuel_cost).sum::<i32>();
    let total_travel_time = stats
        .iter()
        .map(|s| s.travel_time + TimeDelta::seconds(1))
        .sum::<Duration>();

    (stats, total_distance, total_fuel_cost, total_travel_time)
}

#[derive(Debug, Clone)]
pub struct ConnectionDetails {
    pub start: Waypoint,
    pub end: Waypoint,
    pub flight_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: Duration,
}

pub fn get_full_dijkstra(
    waypoints: &HashMap<String, models::Waypoint>,
    start_symbol: String,
    max_fuel: i32,
    nav_mode: NavMode,
    only_markets: bool,
) -> Result<HashMap<String, RouteConnection>, Error> {
    let mut unvisited: HashMap<String, models::Waypoint> = waypoints.clone();
    // .clone()
    // .iter()
    // .map(|w| (w.symbol.clone(), w.clone()))
    // .collect();
    let mut visited: HashMap<String, RouteConnection> = HashMap::new();
    let mut to_visit: PriorityQueue<RouteConnection, Reverse<i64>> = PriorityQueue::new();

    let start = match unvisited.get(&start_symbol) {
        Some(it) => it,
        None => return Err(anyhow::anyhow!("Could not find start waypoint")),
    };

    let modes = nav_mode.get_modes(Modes {
        burn: Mode {
            radius: (max_fuel as f64) / 2.0,
            cost_multiplier: 0.5,
            mode: models::ShipNavFlightMode::Burn,
        },
        cruise: Mode {
            radius: (max_fuel as f64),
            cost_multiplier: 1.0,
            mode: models::ShipNavFlightMode::Cruise,
        },
        drift: Mode {
            radius: f64::INFINITY,
            cost_multiplier: 10.0,
            mode: models::ShipNavFlightMode::Drift,
        },
    });

    to_visit.push(
        RouteConnection {
            start_symbol: "".to_string(),
            end_symbol: start.symbol.clone(),
            distance: 0.0,
            cost: 0.0,
            flight_mode: models::ShipNavFlightMode::Drift,
            re_cost: 0.0,
        },
        Reverse(0),
    );

    while !to_visit.is_empty() {
        let (current_route, _) = to_visit
            .pop()
            .ok_or(anyhow::anyhow!("Could not pop from queue"))?;
        to_visit = to_visit
            .into_iter()
            .filter(|(c, _)| current_route.end_symbol != c.end_symbol)
            .collect();
        visited.insert(current_route.end_symbol.clone(), current_route.clone());
        let current = unvisited
            .remove(&current_route.end_symbol)
            .ok_or(anyhow::anyhow!("Could not remove from queue"))?;

        if !only_markets
            || current
                .traits
                .iter()
                .any(|t| t.symbol == models::WaypointTraitSymbol::Marketplace)
        {
            for mode in &modes {
                let next_waypoints =
                    get_waypoints_within_radius(&unvisited, (current.x, current.y), mode.radius);

                // depends on luck what waypoints are chosen first on same cost
                // next_waypoints.sort_by(|a, b| a.symbol.cmp(&b.symbol));
                // next_waypoints.sort_by(|a, b| b.symbol.cmp(&a.symbol));
                // next_waypoints.reverse();

                for waypoint in next_waypoints.iter() {
                    let distance = distance_between_waypoints(
                        (current.x, current.y),
                        (waypoint.x, waypoint.y),
                    );
                    let cost = current_route.cost + (distance * mode.cost_multiplier) + 1.0;
                    to_visit.push_increase(
                        RouteConnection {
                            start_symbol: current.symbol.clone(),
                            end_symbol: waypoint.symbol.clone(),
                            distance,
                            cost,
                            flight_mode: mode.mode,
                            re_cost: cost,
                        },
                        Reverse((cost * 1000000.0) as i64),
                    );
                }
            }
        }
    }
    Ok(visited)
}

fn get_waypoints_within_radius(
    waypoints: &HashMap<String, models::Waypoint>,
    start_waypoint: (i32, i32),
    radius: f64,
) -> Vec<&models::Waypoint> {
    waypoints
        .iter()
        .filter(|w| {
            let distance = distance_between_waypoints(start_waypoint, (w.1.x, w.1.y));
            distance <= radius
        })
        .map(|w| w.1)
        .collect()
}

#[derive(Debug, Clone)]
pub struct Stat {
    distance: f64,
    fuel_cost: i32,
    travel_time: Duration,
}
pub fn get_inter_system_travel_stats(
    engine_speed: i32,
    flight_mode: models::ShipNavFlightMode,
    start_waypoint: (i32, i32),
    end_waypoint: (i32, i32),
) -> Stat {
    let distance = distance_between_waypoints(start_waypoint, end_waypoint);

    let (fuel_cost, multiplier) = match flight_mode {
        models::ShipNavFlightMode::Burn => (2 * distance as i32, 12),
        models::ShipNavFlightMode::Cruise => (distance as i32, 25),
        models::ShipNavFlightMode::Stealth => (distance as i32, 30),
        models::ShipNavFlightMode::Drift => (1, 250),
    };

    let travel_time = 15.0 + (distance * (multiplier as f64)) / engine_speed as f64;

    Stat {
        distance,
        fuel_cost,
        travel_time: Duration::milliseconds((travel_time * 1000.0) as i64),
    }
}

pub fn distance_between_waypoints_sqrt(
    start_waypoint: (i32, i32),
    end_waypoint: (i32, i32),
) -> f64 {
    let sqrt =
        (end_waypoint.0 - start_waypoint.0).pow(2) + (end_waypoint.1 - start_waypoint.1).pow(2);
    sqrt as f64
}
pub fn distance_between_waypoints(start_waypoint: (i32, i32), end_waypoint: (i32, i32)) -> f64 {
    let sqrt = distance_between_waypoints_sqrt(start_waypoint, end_waypoint).sqrt();
    sqrt
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn get_inter_system_travel_stats_test() {
        let erg =
            get_inter_system_travel_stats(10, models::ShipNavFlightMode::Cruise, (0, 0), (3, 4));

        assert!(
            erg.distance == 5.0
                && erg.fuel_cost == 5
                && erg.travel_time == Duration::milliseconds((27.5 * 1000.0) as i64),
            "erg != 5.0, 5, 27.5 was {:?}",
            erg
        );
    }

    pub fn are_equal_routes(
        waypoints: &HashMap<String, models::Waypoint>,
        a: Vec<RouteConnection>,
        b: Vec<RouteConnection>,
    ) -> bool {
        if a.len() != b.len() {
            return false;
        }
        for i in 0..a.len() {
            if !is_equal_route(waypoints, &a[i], &b[i]) {
                return false;
            }
        }
        true
    }

    pub fn is_equal_route(
        waypoints: &HashMap<String, models::Waypoint>,
        a: &RouteConnection,
        b: &RouteConnection,
    ) -> bool {
        let start_wp_a = waypoints
            .get(&a.start_symbol)
            .expect(format!("Did not find {:?}", a.start_symbol).as_str());
        let end_wp_a = waypoints
            .get(&a.end_symbol)
            .expect(format!("Did not find {:?}", a.end_symbol).as_str());

        let start_wp_b = waypoints
            .get(&b.start_symbol)
            .expect(format!("Did not find {:?}", b.start_symbol).as_str());
        let end_wp_b = waypoints
            .get(&b.end_symbol)
            .expect(format!("Did not find {:?}", b.end_symbol).as_str());

        start_wp_a.x == start_wp_b.x
            && start_wp_a.y == start_wp_b.y
            && end_wp_a.x == end_wp_b.x
            && end_wp_a.y == end_wp_b.y
    }
}
