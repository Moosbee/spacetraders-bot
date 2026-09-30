use std::collections::HashMap;

use space_traders_client::models;
use utils::{WaypointCan, distance_between_waypoints};

pub mod minimum_spanning_tree;

/**
 * System information stuff
 * only MOON and ORBITAL_STATION orbit something else
 * MOON and ORBITAL_STATION always orbit something
 * only PLANET or GAS_GIANT can be orbited
 * the average PLANET has 1,20584069602752 orbitals and up to 8 orbitals
 * the average GAS_GIANT has 2,55359162895928 orbitals and up to 11 orbitals
 * PLANETs generate from 7 to 246, one found at 252
 * GAS_GIANTs generate from 7 to 288 and from 406 to 413
 * ASTEROID_BASEs generate from 296 to 303, from 342 to 347 and from 716 to 722
 * ENGINEERED_ASTEROIDs generate from 24 to 30
 * JUMP_GATEs generate from 446 to 453
 * FUEL_STATIONs generate from 112 to 117, from 186 to 203, from 226 to 233, from 297 to 303 and from 596 to 602
 * systems can have two asteroid belts
 * the inner belt is from 307 to 392
 * the outer belt is from 707 to 792
 * systems only have at most one jump gate
 *
 */

pub fn check_waypoint(waypoint: &database::Waypoint) -> bool {
    match &waypoint.waypoint_type {
        models::WaypointType::Moon => waypoint.orbits.is_some() && waypoint.orbitals.is_empty(),
        models::WaypointType::OrbitalStation => {
            waypoint.orbits.is_some() && waypoint.orbitals.is_empty()
        }
        models::WaypointType::Planet => {
            let distance = distance_between_waypoints((0, 0), (waypoint.x, waypoint.y));
            waypoint.orbits.is_none() && (distance >= 7.0 && distance <= 253.0)
        }
        models::WaypointType::GasGiant => {
            let distance = distance_between_waypoints((0, 0), (waypoint.x, waypoint.y));

            waypoint.orbits.is_none()
                && ((distance >= 7.0 && distance <= 289.0)
                    || (distance >= 406.0 && distance <= 414.0))
        }
        models::WaypointType::JumpGate => {
            let distance = distance_between_waypoints((0, 0), (waypoint.x, waypoint.y));

            waypoint.orbits.is_none()
                && waypoint.orbitals.is_empty()
                && (distance >= 446.0 && distance <= 454.0)
        }
        models::WaypointType::Asteroid => {
            let distance = distance_between_waypoints((0, 0), (waypoint.x, waypoint.y));

            waypoint.orbits.is_none()
                && waypoint.orbitals.is_empty()
                && ((distance >= 307.0 && distance <= 393.0)
                    || (distance >= 707.0 && distance <= 793.0))
        }
        models::WaypointType::EngineeredAsteroid => {
            let distance = distance_between_waypoints((0, 0), (waypoint.x, waypoint.y));

            waypoint.orbits.is_none()
                && waypoint.orbitals.is_empty()
                && (distance >= 24.0 && distance <= 31.0)
        }
        models::WaypointType::AsteroidBase => {
            let distance = distance_between_waypoints((0, 0), (waypoint.x, waypoint.y));
            waypoint.orbits.is_none()
                && waypoint.orbitals.is_empty()
                && ((distance >= 296.0 && distance <= 304.0)
                    || (distance >= 342.0 && distance <= 348.0)
                    || (distance >= 716.0 && distance <= 723.0))
        }
        models::WaypointType::FuelStation => {
            let distance = distance_between_waypoints((0, 0), (waypoint.x, waypoint.y));
            waypoint.orbits.is_none()
                && waypoint.orbitals.is_empty()
                && ((distance >= 112.0 && distance <= 118.0)
                    || (distance >= 186.0 && distance <= 204.0)
                    || (distance >= 226.0 && distance <= 234.0)
                    || (distance >= 296.0 && distance <= 304.0)
                    || (distance >= 596.0 && distance <= 603.0))
        }
        models::WaypointType::AsteroidField => false,
        models::WaypointType::Nebula => false,
        models::WaypointType::DebrisField => false,
        models::WaypointType::GravityWell => false,
        models::WaypointType::ArtificialGravityWell => false,
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
pub struct SystemAnalyzation {
    pub total_waypoint_count: u32, // total count of waypoints in system
    pub total_market_count: u32,   // total count of market waypoints in system
    pub has_jump_gate: bool,       // true if system has jump gate
    pub fuel_station_count: u32,   // total count of fuel_stations in system
    pub inner_system_count: u32,   // total count of waypoints in inner system distance < 300
    pub inner_system_planet_count: u32, // total count of planets in inner system distance < 300
    pub inner_system_planet_orbitals_count: u32, // total count of waypoints in inner system that orbit planets distance < 300
    pub inner_system_gas_giant_count: u32, // total count of gas giants in inner system distance < 300
    pub inner_system_gas_giant_orbitals_count: u32, // total count of waypoints in inner system that orbit gas giants distance < 300
    pub inner_system_moon_count: u32, // total count of moons in inner system distance < 300
    pub inner_system_orbital_station_count: u32, // total count of orbital stations in inner system distance < 300
    pub first_asteroid_base_count: u32,          // count of asteroid bases between 296 to 303
    pub second_asteroid_base_count: u32,         // count of asteroid bases between 342 to 348
    pub third_asteroid_base_count: u32,          // count of asteroid bases between 716 to 723
    pub first_asteroid_count: u32,               // count of asteroid belts between 307 to 393
    pub second_asteroid_count: u32,              // count of asteroid belts between 707 to 793
    pub outer_system_gas_giant_count: u32, // total count of gas giants in outer system distance > 300
    pub outer_system_gas_giant_orbitals_count: u32, // total count of waypoints in outer system that orbit gas giants distance > 300
    pub outer_system_moon_count: u32, // total count of moons in outer system distance > 300
    pub outer_system_orbital_station_count: u32, // total count of orbital stations in outer system distance > 300
}

pub fn gen_system_analyzation(waypoints: &[database::Waypoint]) -> SystemAnalyzation {
    let mut analyzation = SystemAnalyzation::default();

    for waypoint in waypoints {
        analyzation.total_waypoint_count += 1;
        if waypoint.is_marketplace() {
            analyzation.total_market_count += 1;
        }
        if waypoint.is_jump_gate() {
            analyzation.has_jump_gate = true;
        }
        if waypoint.waypoint_type == models::WaypointType::FuelStation {
            analyzation.fuel_station_count += 1;
        }

        let distance = distance_between_waypoints((0, 0), (waypoint.x, waypoint.y));

        if distance < 300.0 {
            analyzation.inner_system_count += 1;
            if waypoint.waypoint_type == models::WaypointType::Planet {
                analyzation.inner_system_planet_count += 1;
                analyzation.inner_system_planet_orbitals_count += waypoint.orbitals.len() as u32;
            }
            if waypoint.waypoint_type == models::WaypointType::GasGiant {
                analyzation.inner_system_gas_giant_count += 1;
                analyzation.inner_system_gas_giant_orbitals_count += waypoint.orbitals.len() as u32;
            }
            if waypoint.waypoint_type == models::WaypointType::Moon {
                analyzation.inner_system_moon_count += 1;
            }
            if waypoint.waypoint_type == models::WaypointType::OrbitalStation {
                analyzation.inner_system_orbital_station_count += 1;
            }
        } else if distance > 300.0 {
            if waypoint.waypoint_type == models::WaypointType::GasGiant {
                analyzation.outer_system_gas_giant_count += 1;
                analyzation.outer_system_gas_giant_orbitals_count += waypoint.orbitals.len() as u32;
            }
            if waypoint.waypoint_type == models::WaypointType::Moon {
                analyzation.outer_system_moon_count += 1;
            }
            if waypoint.waypoint_type == models::WaypointType::OrbitalStation {
                analyzation.outer_system_orbital_station_count += 1;
            }
        }

        if waypoint.waypoint_type == models::WaypointType::AsteroidBase {
            if distance > 296.0 && distance <= 303.0 {
                analyzation.first_asteroid_base_count += 1;
            } else if distance > 342.0 && distance <= 348.0 {
                analyzation.second_asteroid_base_count += 1;
            } else if distance > 716.0 && distance <= 723.0 {
                analyzation.third_asteroid_base_count += 1;
            }
        }

        if waypoint.waypoint_type == models::WaypointType::Asteroid {
            if distance > 307.0 && distance <= 393.0 {
                analyzation.first_asteroid_count += 1;
            } else if distance > 707.0 && distance <= 793.0 {
                analyzation.second_asteroid_count += 1;
            }
        }
    }

    analyzation
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
