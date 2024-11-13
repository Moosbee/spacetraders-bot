use std::collections::HashMap;

use anyhow::Error;
use space_traders_client::models;

use super::nav_models::RouteConnection;

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

pub fn get_route(
    visited: HashMap<String, RouteConnection>,
    start_symbol: String,
    end_symbol: String,
) -> Result<Vec<RouteConnection>, Error> {
    let mut route = Vec::new();
    let mut current = end_symbol;
    while current != start_symbol {
        let connection = visited
            .get(&current)
            .ok_or_else(|| anyhow::anyhow!("Could not find connection"))?;
        route.push(connection.clone());
        current = connection.start_symbol.clone();
    }
    route.reverse();
    Ok(route)
}

pub fn distance_between_waypoints(start: (i32, i32), end: (i32, i32)) -> f64 {
    (((end.0 - start.0).pow(2) + (end.1 - start.1).pow(2)) as f64).sqrt()
}

pub fn get_nearby_waypoints<'a>(
    waypoints: &'a HashMap<String, models::Waypoint>,
    start_waypoint: (i32, i32),
    radius: f64,
) -> Vec<&'a models::Waypoint> {
    waypoints
        .values()
        .filter(|w| distance_between_waypoints(start_waypoint, (w.x, w.y)) <= radius)
        .collect()
}
