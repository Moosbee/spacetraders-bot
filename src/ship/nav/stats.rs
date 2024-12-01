use crate::IsMarketplace;

use super::{
    nav_models::{ConnectionDetails, RouteConnection, RouteInstruction},
    utils::*,
};
use chrono::{Duration, TimeDelta};
use space_traders_client::models;
use std::collections::HashMap;

pub fn calc_route_stats(
    waypoints: &HashMap<String, models::Waypoint>,
    route: &[RouteConnection],
    engine_speed: i32,
) -> (Vec<ConnectionDetails>, f64, i32, TimeDelta) {
    let stats: Vec<_> = route
        .iter()
        .map(|conn| calculate_connection_details(waypoints, conn, engine_speed))
        .collect();

    let total_distance = stats.iter().map(|s| s.distance).sum();
    let total_fuel_cost = stats.iter().map(|s| s.fuel_cost).sum();
    let total_travel_time = stats
        .iter()
        .map(|s| s.travel_time + TimeDelta::seconds(1))
        .sum();

    (stats, total_distance, total_fuel_cost, total_travel_time)
}

fn calculate_connection_details(
    waypoints: &HashMap<String, models::Waypoint>,
    conn: &RouteConnection,
    engine_speed: i32,
) -> ConnectionDetails {
    let start = waypoints.get(&conn.start_symbol).unwrap();
    let end = waypoints.get(&conn.end_symbol).unwrap();
    let stat = get_travel_stats(
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
}

#[derive(Debug, Clone)]
pub struct TravelStats {
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: Duration,
}

pub fn get_travel_stats(
    engine_speed: i32,
    flight_mode: models::ShipNavFlightMode,
    start: (i32, i32),
    end: (i32, i32),
) -> TravelStats {
    let distance = distance_between_waypoints(start, end);
    let (fuel_cost, multiplier) = calculate_fuel_and_multiplier(flight_mode, distance);
    let travel_time = calculate_travel_time(distance, multiplier, engine_speed);

    TravelStats {
        distance,
        fuel_cost,
        travel_time: Duration::milliseconds((travel_time * 1000.0) as i64),
    }
}

fn calculate_fuel_and_multiplier(
    flight_mode: models::ShipNavFlightMode,
    distance: f64,
) -> (i32, i32) {
    match flight_mode {
        models::ShipNavFlightMode::Burn => ((2.0 * distance.max(1.0)).round() as i32, 12),
        models::ShipNavFlightMode::Cruise => ((distance.max(1.0)).round() as i32, 25),
        models::ShipNavFlightMode::Stealth => ((distance.max(1.0)).round() as i32, 30),
        models::ShipNavFlightMode::Drift => (1, 250),
    }
}

fn calculate_travel_time(distance: f64, multiplier: i32, engine_speed: i32) -> f64 {
    let result = ((1.0_f64.max(distance).round()) * ((multiplier as f64) / (engine_speed as f64)))
        .round()
        + 15.0;
    result
}

pub fn generate_route_instructions(route: Vec<ConnectionDetails>) -> Vec<RouteInstruction> {
    let mut instructions = Vec::new();
    let mut last_fuel_cap = 0;

    for conn in route.iter().rev() {
        let start_is_marketplace = conn.start.is_marketplace();

        if !start_is_marketplace {
            last_fuel_cap += conn.fuel_cost;
        }

        instructions.push(RouteInstruction {
            start_symbol: conn.start.symbol.clone(),
            end_symbol: conn.end.symbol.clone(),
            start_is_marketplace,
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
