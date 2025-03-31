use space_traders_client::models;

use crate::utils::distance_between_waypoints;

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
    distance: f64,
) -> TravelStats {
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
