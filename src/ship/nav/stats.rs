use crate::{sql, types::WaypointCan};

use super::{
    nav_models::{ConnectionDetails, RouteConnection, RouteInstruction},
    utils::*,
};
use log::debug;
use space_traders_client::models;
use std::collections::HashMap;

/// Calculate various statistics for a given route.
///
/// This function calculates the total distance, total fuel cost, and total travel time
/// for a given route. The route is given as an array of `RouteConnection`s, and the
/// engine speed, engine condition, frame condition, and reactor condition are given as
/// arguments.
///
/// The function returns a tuple of four elements. The first element is a vector of
/// `ConnectionDetails` which contains the detailed statistics for each connection in
/// the route. The second element is the total distance of the route. The third element
/// is the total fuel cost of the route. The fourth element is the total travel time of
/// the route.
///
/// The total travel time is calculated by summing the travel times of all the
/// connections in the route, and adding 1 to each travel time to account for the
/// time it takes to transition between connections.
pub fn calc_route_stats(
    waypoints: &HashMap<String, sql::Waypoint>,
    route: &[RouteConnection],
    engine_speed: i32,
    engine_condition: f64,
    frame_condition: f64,
    reactor_condition: f64,
) -> (Vec<ConnectionDetails>, f64, i32, f64) {
    let stats: Vec<_> = route
        .iter()
        .map(|conn| {
            calculate_connection_details(
                waypoints,
                conn,
                engine_speed,
                engine_condition,
                frame_condition,
                reactor_condition,
            )
        })
        .collect();

    let total_distance = stats.iter().map(|s| s.distance).sum();
    let total_fuel_cost = stats.iter().map(|s| s.fuel_cost).sum();
    let total_travel_time = stats
        .iter()
        .map(|s| {
            let time = s.travel_time + 1.0;
            debug!("Travel time: {:?} from {:?}", time, s.travel_time);
            time
        })
        .fold(0.0, |a, b| a + b);

    debug!("Total travel time: {:?}", total_travel_time);

    (stats, total_distance, total_fuel_cost, total_travel_time)
}

fn calculate_connection_details(
    waypoints: &HashMap<String, sql::Waypoint>,
    conn: &RouteConnection,
    engine_speed: i32,
    engine_condition: f64,
    frame_condition: f64,
    reactor_condition: f64,
) -> ConnectionDetails {
    let start = waypoints.get(&conn.start_symbol).unwrap();
    let end = waypoints.get(&conn.end_symbol).unwrap();
    let stat = get_travel_stats(
        engine_speed,
        conn.flight_mode,
        engine_condition,
        frame_condition,
        reactor_condition,
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
}

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
    frame_condition: f64,
    reactor_condition: f64,
    start: (i32, i32),
    end: (i32, i32),
) -> TravelStats {
    let distance = distance_between_waypoints(start, end);
    let (fuel_cost, multiplier) = calculate_fuel_and_multiplier(flight_mode, distance);
    let travel_time = calculate_travel_time(
        distance,
        multiplier,
        engine_speed,
        engine_condition,
        frame_condition,
        reactor_condition,
    );

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

fn calculate_travel_time(
    distance: f64,
    multiplier: f64,
    engine_speed: i32,
    engine_condition: f64,
    frame_condition: f64,
    reactor_condition: f64,
) -> f64 {
    ((distance.max(1.0).round()) * (multiplier / (engine_speed as f64)) + 15.0).round()
}

/// Generates a set of route instructions based on a route of ConnectionDetails.
///
/// The instructions are generated in reverse order of the route, and the
/// `refuel_to` and `fuel_in_cargo` fields are filled in assuming that the
/// ship is initially full of fuel (if `prepare` is true) and then
/// incrementally updates its fuel capacity based on the fuel cost of each
/// leg of the route.  If a leg of the route starts at a marketplace, the
/// fuel capacity is reset to 0.
pub fn generate_route_instructions(
    route: Vec<ConnectionDetails>,
    prepare: bool,
) -> Vec<RouteInstruction> {
    let mut instructions = Vec::new();
    let mut last_fuel_cap = (prepare as i32) * 100;

    for conn in route.iter().rev() {
        let start_is_marketplace = conn.start.is_marketplace();

        if !start_is_marketplace {
            last_fuel_cap += conn.fuel_cost;
        }

        instructions.push(RouteInstruction {
            start_symbol: conn.start.symbol.clone(),
            end_symbol: conn.end.symbol.clone(),
            start_is_marketplace,
            distance: conn.distance,
            flight_mode: conn.flight_mode,
            refuel_to: conn.fuel_cost,
            fuel_in_cargo: last_fuel_cap,
        });

        if start_is_marketplace {
            last_fuel_cap = 0;
        }
    }

    instructions.reverse();
    instructions
}
