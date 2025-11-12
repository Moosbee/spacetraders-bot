use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use database::DatabaseConnector;
use priority_queue::PriorityQueue;

use crate::error::Result;

#[derive(Clone, Debug)]
pub struct GateConnection {
    pub point_a: String,
    pub pos_point_a: (i32, i32),
    pub system_point_a: String,
    pub point_b: String,
    pub pos_point_b: (i32, i32),
    pub system_point_b: String,
    pub under_construction_a: bool,
    pub under_construction_b: bool,
    pub from_a: bool,
    pub from_b: bool,
    pub distance: f64,
}

impl Hash for GateConnection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.point_a.hash(state);
        self.pos_point_a.hash(state);
        self.system_point_a.hash(state);
        self.point_b.hash(state);
        self.pos_point_b.hash(state);
        self.system_point_b.hash(state);
        self.under_construction_a.hash(state);
        self.under_construction_b.hash(state);
        self.from_a.hash(state);
        self.from_b.hash(state);
    }
}

impl PartialEq for GateConnection {
    fn eq(&self, other: &Self) -> bool {
        self.point_a == other.point_a
            && self.pos_point_a == other.pos_point_a
            && self.system_point_a == other.system_point_a
            && self.point_b == other.point_b
            && self.pos_point_b == other.pos_point_b
            && self.system_point_b == other.system_point_b
            && self.under_construction_a == other.under_construction_a
            && self.under_construction_b == other.under_construction_b
            && self.from_a == other.from_a
            && self.from_b == other.from_b
    }
}

impl Eq for GateConnection {}

impl GateConnection {
    pub fn get_other(&self, point: &str) -> (String, String) {
        if self.point_a == point {
            (self.point_b.clone(), self.system_point_b.clone())
        } else {
            (self.point_a.clone(), self.system_point_a.clone())
        }
    }
    pub fn get_other_system(&self, system_point: &str) -> (String, String) {
        if self.system_point_a == system_point {
            (self.point_b.clone(), self.system_point_b.clone())
        } else {
            (self.point_a.clone(), self.system_point_a.clone())
        }
    }
}

#[derive(Debug, Clone)]
pub struct JumpConnection {
    pub start_system: String,
    pub end_system: String,
    pub conn: GateConnection,
    pub cost: f64,
}

impl Eq for JumpConnection {}

impl PartialEq for JumpConnection {
    fn eq(&self, other: &Self) -> bool {
        self.start_system == other.start_system
            && self.end_system == other.end_system
            && self.conn == other.conn
    }
}

impl Hash for JumpConnection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start_system.hash(state);
        self.end_system.hash(state);
        self.conn.hash(state);
    }
}

pub struct JumpPathfinder {
    all_connections: Vec<GateConnection>,
}

impl JumpPathfinder {
    pub fn new(all_connections: Vec<GateConnection>) -> Self {
        Self { all_connections }
    }

    pub fn find_route(&self, from_system: &str, to_system: &str) -> Vec<JumpConnection> {
        let mut unvisited: Vec<GateConnection> = self.all_connections.clone();
        let mut to_visit: PriorityQueue<JumpConnection, Reverse<i64>> = PriorityQueue::new();
        let mut visited: HashMap<String, JumpConnection> = HashMap::new();

    // tracing::info!(from_system = %from_system, to_system = %to_system, "Finding route");

        let start_conns = Self::get_connections(from_system, &mut unvisited);
        for conn in start_conns {
            let gate = JumpConnection {
                start_system: from_system.to_string(),
                end_system: conn.get_other_system(from_system).1,
                cost: conn.distance,
                conn,
            };
            let next_cost = Reverse((gate.cost * 1_000_000.0) as i64);
            to_visit.push(gate, next_cost);
        }

        while let Some((conn, _)) = to_visit.pop() {
            visited.insert(conn.end_system.clone(), conn.clone());
            if conn.end_system == to_system {
                return Self::get_route(visited, from_system.to_string(), to_system.to_string());
            }
            let conns = Self::get_connections(&conn.end_system, &mut unvisited);
            for next_conn in conns {
                let next_cost = conn.cost + next_conn.distance;
                let next_conn = JumpConnection {
                    start_system: conn.end_system.to_string(),
                    end_system: next_conn.get_other_system(&conn.end_system).1,
                    conn: next_conn,
                    cost: next_cost,
                };

                to_visit.push(next_conn, Reverse((next_cost * 1_000_000.0) as i64));
            }
        }

        vec![]
    }

    fn get_connections(
        from_system: &str,
        unvisited: &mut Vec<GateConnection>,
    ) -> Vec<GateConnection> {
        let conns = unvisited
            .iter()
            .filter(|conn| conn.system_point_a == from_system || conn.system_point_b == from_system)
            .cloned()
            .collect::<Vec<_>>();
        unvisited.retain(|conn| {
            !(conn.system_point_a == from_system || conn.system_point_b == from_system)
        });
        conns
    }

    fn get_route(
        visited: HashMap<String, JumpConnection>,
        from: String,
        to_string: String,
    ) -> Vec<JumpConnection> {
        let mut route = Vec::new();
        let mut current = to_string.clone();
    tracing::debug!(visited_count = %visited.len(), "Visited systems");
        while current != from {
            let connection = visited.get(&current).unwrap();
            route.push(connection.clone());
            current = connection.start_system.clone();
        }
        route.reverse();
    tracing::debug!(route = ?route, "Route calculated");
        route
    }
}

pub async fn generate_all_connections(
    database_pool: &database::DbPool,
) -> Result<Vec<GateConnection>> {
    let all_connections = database::JumpGateConnection::get_all(database_pool).await?;

    let mut connection_map: HashMap<(String, String), GateConnection> = HashMap::new();

    for connection in all_connections {
        let mut pair = [connection.from.clone(), connection.to.clone()];
        pair.sort(); // Ensure the pair is always in a consistent order
        let entry = connection_map.entry((pair[0].clone(), pair[1].clone()));

        let entry = entry.or_insert_with(|| GateConnection {
            point_a: pair[0].clone(),
            point_b: pair[1].clone(),
            under_construction_a: false,
            under_construction_b: false,
            from_a: false,
            from_b: false,
            pos_point_a: (0, 0),
            system_point_a: String::new(),
            pos_point_b: (0, 0),
            system_point_b: String::new(),
            distance: 0.0,
        });
        let is_from_a = connection.from == pair[0];
        let is_from_b = connection.from == pair[1];
        if is_from_a {
            entry.from_a = true;
        } else if is_from_b {
            entry.from_b = true;
        }
    }

    let mut waypoints = HashMap::new();
    for waypoint in connection_map
        .keys()
        .flat_map(|k| [k.0.clone(), k.1.clone()])
        .collect::<HashSet<_>>()
    {
        let wp = database::Waypoint::get_by_symbol(database_pool, &waypoint).await?;
        if let Some(wp) = wp {
            waypoints.insert(waypoint, wp);
        }
    }

    let mut systems = HashMap::new();
    for waypoint in waypoints.values() {
        let system = database::System::get_by_id(database_pool, &waypoint.system_symbol).await?;
        if let Some(system) = system {
            systems.insert(waypoint.system_symbol.clone(), system);
        }
    }

    let connections = connection_map
        .into_values()
        .map(|mut c| {
            let wp_a = waypoints.get(&c.point_a).unwrap();
            let wp_b = waypoints.get(&c.point_b).unwrap();
            let sys_a = systems.get(&wp_a.system_symbol).unwrap();
            let sys_b = systems.get(&wp_b.system_symbol).unwrap();

            c.pos_point_a = (sys_a.x, sys_a.y);
            c.pos_point_b = (sys_b.x, sys_b.y);
            c.system_point_a = wp_a.system_symbol.clone();
            c.system_point_b = wp_b.system_symbol.clone();

            let distance = ((c.pos_point_a.0 - c.pos_point_b.0).pow(2)
                + (c.pos_point_a.1 - c.pos_point_b.1).pow(2)) as f64;
            c.distance = distance.sqrt();
            c.under_construction_a = wp_a.is_under_construction;
            c.under_construction_b = wp_b.is_under_construction;

            c
        })
        .collect::<Vec<_>>();

    Ok(connections)
}
