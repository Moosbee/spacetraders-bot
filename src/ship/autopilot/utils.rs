use std::collections::HashMap;

use crate::{sql, utils::distance_between_waypoints};

pub fn get_nearby_waypoints(
    waypoints: &HashMap<String, sql::Waypoint>,
    start_waypoint: (i32, i32),
    radius: f64,
) -> Vec<&sql::Waypoint> {
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
