use std::collections::HashMap;

use utils::distance_between_waypoints;

pub fn get_nearby_waypoints(
    waypoints: &HashMap<String, database::Waypoint>,
    start_waypoint: (i32, i32),
    radius: f64,
) -> Vec<&database::Waypoint> {
    waypoints
        .values()
        .filter(|w| distance_between_waypoints(start_waypoint, (w.x, w.y)) <= radius)
        .collect()
}

pub(crate) fn get_route(
    visited: HashMap<String, super::connection::SimpleConnection>,
    start_symbol: String,
    end_symbol: String,
) -> Option<Vec<super::connection::SimpleConnection>> {
    let mut route = Vec::new();
    let mut current = end_symbol.clone();
    while current != start_symbol {
        let connection = visited.get(&current)?;
        route.push(connection.clone());
        current = connection.start_symbol.clone();
    }
    route.reverse();
    Some(route)
}

pub fn estimate_route_cost(
    route: &[super::connection::SimpleConnection],
    fuel_cost: i64,
    antimatter_cost: i64,
) -> i64 {
    route.iter().fold(0, |acc, conn| {
        acc + match conn.connection_type {
            super::connection::ConnectionType::JumpGate => antimatter_cost,
            super::connection::ConnectionType::Warp { .. } => {
                fuel_cost * (conn.distance / 10.0).ceil() as i64
            }
            super::connection::ConnectionType::Navigate { .. } => {
                fuel_cost * (conn.distance / 10.0).ceil() as i64
            }
        }
    })
}
