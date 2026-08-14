use std::collections::HashMap;

use utils::distance_between_waypoints;

pub fn get_nearby_waypoints_donut(
    waypoints: &HashMap<String, database::Waypoint>,
    start_waypoint: (i32, i32),
    min_radius: f64,
    max_radius: f64,
) -> Vec<&database::Waypoint> {
    waypoints
        .values()
        .filter(|w| {
            let distance = distance_between_waypoints(start_waypoint, (w.x, w.y));
            distance <= max_radius && distance > min_radius
        })
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

use space_traders_client::models;

#[derive(Debug, Clone)]
pub struct TravelStats {
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: f64,
}

pub fn get_travel_stats(
    engine_speed: i32,
    flight_mode: models::ShipNavFlightMode,
    engine_condition: f64,
    distance: f64,
) -> TravelStats {
    let (fuel_cost, multiplier) = calculate_fuel_and_multiplier(flight_mode, distance);
    let travel_time = calculate_travel_time(distance, multiplier, engine_speed, engine_condition);

    TravelStats {
        distance,
        fuel_cost,
        travel_time,
    }
}

fn calculate_fuel_and_multiplier(
    flight_mode: models::ShipNavFlightMode,
    distance: f64,
) -> (i32, f64) {
    match flight_mode {
        models::ShipNavFlightMode::Burn => ((2.0 * distance.max(1.0)).ceil() as i32, 12.5),
        models::ShipNavFlightMode::Cruise => ((distance.max(1.0)).ceil() as i32, 25.0),
        models::ShipNavFlightMode::Stealth => ((distance.max(1.0)).ceil() as i32, 30.0),
        models::ShipNavFlightMode::Drift => (1, 250.0),
    }
}

pub fn calculate_jump_cooldown(distance: f64) -> f64 {
    (15.0 + 0.3 * distance).round()
}

fn calculate_travel_time(
    distance: f64,
    multiplier: f64,
    engine_speed: i32,
    _engine_condition: f64,
) -> f64 {
    ((distance.max(1.0).round()) * (multiplier / (engine_speed as f64)) + 15.0).round()
}
