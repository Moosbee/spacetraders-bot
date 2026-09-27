use std::collections::HashMap;

use space_traders_client::models;
use utils::distance_between_waypoints;

mod minimum_spanning_tree;

pub struct SystemAnalyzation {
    pub waypoint_count: i32,     // total count of waypoints in system
    pub market_count: i32,       // total count of market waypoints in system
    pub has_jump_gate: bool,     // true if system has jump gate
    pub fuel_station_count: i32, // total count of fuel_stations in system
    pub inner_system_count: i32, // total count of waypoints in inner system distance < 300
    pub inner_system_market_count: i32,
    pub first_asteroid_belt_count: i32,
    pub second_asteroid_belt_count: i32,
    pub other_asteroids_count: i32,
}

pub struct WaypointCluster<'a> {
    x: i32,
    y: i32,
    waypoints: Vec<&'a database::Waypoint>,
}

pub fn get_waypoint_clusters<'a>(
    waypoints: &'a [database::Waypoint],
) -> HashMap<(i32, i32), WaypointCluster<'a>> {
    let mut clusters = HashMap::new();
    for waypoint in waypoints {
        let cluster = clusters
            .entry((waypoint.x, waypoint.y))
            .or_insert(WaypointCluster {
                x: waypoint.x,
                y: waypoint.y,
                waypoints: Vec::new(),
            });
        cluster.waypoints.push(waypoint);
    }
    clusters
}

pub struct AsteroidBelts<'a> {
    first: Vec<&'a database::Waypoint>, // first one is from distance 300 to 400
    second: Vec<&'a database::Waypoint>, // second one is from distance 700 to 800
    other: Vec<&'a database::Waypoint>,
}

pub fn get_asteroid_belts<'a>(waypoints: &'a [database::Waypoint]) -> AsteroidBelts<'a> {
    let mut asteroid_belts = AsteroidBelts {
        first: Vec::new(),
        second: Vec::new(),
        other: Vec::new(),
    };

    for waypoint in waypoints
        .iter()
        .filter(|f| f.waypoint_type == models::WaypointType::Asteroid)
    {
        let distance = distance_between_waypoints((0, 0), (waypoint.x, waypoint.y));

        if distance > 300.0 && distance <= 400.0 {
            asteroid_belts.first.push(waypoint);
        } else if distance > 700.0 && distance <= 800.0 {
            asteroid_belts.second.push(waypoint);
        } else {
            asteroid_belts.other.push(waypoint);
        }
    }

    asteroid_belts
}
