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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use space_traders_client::models::ShipNavFlightMode;

    use super::super::connection::{ConnectionType, SimpleConnection};
    use super::{get_nearby_waypoints_donut, get_route};

    fn waypoint(symbol: &str, x: i32, y: i32) -> database::Waypoint {
        database::Waypoint {
            symbol: symbol.to_string(),
            x,
            y,
            ..Default::default()
        }
    }

    fn connection(start: &str, end: &str) -> SimpleConnection {
        SimpleConnection {
            start_symbol: start.to_string(),
            end_symbol: end.to_string(),
            connection_type: ConnectionType::Navigate {
                nav_mode: ShipNavFlightMode::Cruise,
            },
            start_is_marketplace: false,
            end_is_marketplace: false,
            cost: 0.0,
            re_cost: 0.0,
            distance: 0.0,
        }
    }

    fn sorted_symbols(waypoints: Vec<&database::Waypoint>) -> Vec<String> {
        let mut symbols: Vec<String> = waypoints.into_iter().map(|w| w.symbol.clone()).collect();
        symbols.sort();
        symbols
    }

    #[test]
    fn donut_keeps_only_waypoints_inside_ring() {
        let waypoints = [
            waypoint("A", 0, 0), // distance 0
            waypoint("B", 3, 4), // distance 5
            waypoint("C", 6, 8), // distance 10
            waypoint("D", 0, 1), // distance 1
        ]
        .into_iter()
        .map(|w| (w.symbol.clone(), w))
        .collect();

        // min_radius is exclusive, max_radius is inclusive.
        assert_eq!(
            sorted_symbols(get_nearby_waypoints_donut(&waypoints, (0, 0), 1.0, 5.0)),
            vec!["B"]
        );
        assert_eq!(
            sorted_symbols(get_nearby_waypoints_donut(&waypoints, (0, 0), 5.0, 10.0)),
            vec!["C"]
        );
    }

    #[test]
    fn donut_negative_min_radius_includes_origin() {
        let waypoints = [
            waypoint("A", 0, 0), // distance 0
            waypoint("B", 0, 1), // distance 1
        ]
        .into_iter()
        .map(|w| (w.symbol.clone(), w))
        .collect();

        assert_eq!(
            sorted_symbols(get_nearby_waypoints_donut(&waypoints, (0, 0), -1.0, 0.0)),
            vec!["A"]
        );
    }

    #[test]
    fn get_route_rebuilds_path_in_order() {
        let mut visited = HashMap::new();
        visited.insert("C".to_string(), connection("B", "C"));
        visited.insert("B".to_string(), connection("A", "B"));

        let route = get_route(visited, "A".to_string(), "C".to_string()).unwrap();

        let path: Vec<(&str, &str)> = route
            .iter()
            .map(|c| (c.start_symbol.as_str(), c.end_symbol.as_str()))
            .collect();
        assert_eq!(path, vec![("A", "B"), ("B", "C")]);
    }

    #[test]
    fn get_route_returns_none_for_broken_chain() {
        let mut visited = HashMap::new();
        visited.insert("C".to_string(), connection("B", "C"));

        assert!(get_route(visited, "A".to_string(), "C".to_string()).is_none());
    }

    #[test]
    fn get_route_to_self_is_empty() {
        let visited = HashMap::new();
        assert_eq!(
            get_route(visited, "A".to_string(), "A".to_string()),
            Some(Vec::new())
        );
    }
}
