use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use database::DatabaseConnectorAsync;
use priority_queue::PriorityQueue;

use tracing::debug;

use crate::error::Result;

#[derive(Clone, Debug)]
pub struct GateConnection {
    pub jump_gate_waypoint_a: String,
    pub system_pos_a: (i32, i32),
    pub system_a: String,
    pub jump_gate_waypoint_b: String,
    pub system_pos_b: (i32, i32),
    pub system_b: String,
    pub under_construction_a: bool,
    pub under_construction_b: bool,
    pub from_a: bool,
    pub from_b: bool,
    pub distance: f64,
}

impl Hash for GateConnection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.jump_gate_waypoint_a.hash(state);
        self.system_pos_a.hash(state);
        self.system_a.hash(state);
        self.jump_gate_waypoint_b.hash(state);
        self.system_pos_b.hash(state);
        self.system_b.hash(state);
        self.under_construction_a.hash(state);
        self.under_construction_b.hash(state);
        self.from_a.hash(state);
        self.from_b.hash(state);
    }
}

impl PartialEq for GateConnection {
    fn eq(&self, other: &Self) -> bool {
        self.jump_gate_waypoint_a == other.jump_gate_waypoint_a
            && self.system_pos_a == other.system_pos_a
            && self.system_a == other.system_a
            && self.jump_gate_waypoint_b == other.jump_gate_waypoint_b
            && self.system_pos_b == other.system_pos_b
            && self.system_b == other.system_b
            && self.under_construction_a == other.under_construction_a
            && self.under_construction_b == other.under_construction_b
            && self.from_a == other.from_a
            && self.from_b == other.from_b
    }
}

impl Eq for GateConnection {}

impl GateConnection {
    pub fn get_other(&self, point: &str) -> (String, String) {
        if self.jump_gate_waypoint_a == point {
            (self.jump_gate_waypoint_b.clone(), self.system_b.clone())
        } else {
            (self.jump_gate_waypoint_a.clone(), self.system_a.clone())
        }
    }
    pub fn get_other_system(&self, system_point: &str) -> (String, String) {
        if self.system_a == system_point {
            (self.jump_gate_waypoint_b.clone(), self.system_b.clone())
        } else {
            (self.jump_gate_waypoint_a.clone(), self.system_a.clone())
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

#[derive(Debug, Clone)]
pub struct JumpGateRouterCache {
    cache: HashMap<(String, String, bool), Vec<JumpConnection>>,
    router: JumpGateRouter,
}

impl JumpGateRouterCache {
    pub fn new(router: JumpGateRouter) -> Self {
        Self {
            cache: HashMap::new(),
            router,
        }
    }

    pub fn find_jump_route<'a>(
        &'a mut self,
        from_system: &str,
        to_system: &str,
        no_under_construction: bool,
    ) -> Option<&'a [JumpConnection]> {
        if !self.cache.contains_key(&(
            from_system.to_string(),
            to_system.to_string(),
            no_under_construction,
        )) {
            debug!(
                key = ?(from_system.to_string(), to_system.to_string(), no_under_construction),
                "Jump Route Cache miss"
            );
            let route = self
                .router
                .find_jump_route(from_system, to_system, no_under_construction);
            let route = route?;
            self.cache.insert(
                (
                    from_system.to_string(),
                    to_system.to_string(),
                    no_under_construction,
                ),
                route,
            );
        }

        Some(self.cache
            .get(&(from_system.to_string(), to_system.to_string(), no_under_construction))
            .unwrap_or_else(|| {
                panic!(
                    "We should have importet the route in the cache(find_cached_route) key: {:?}",
                    (from_system.to_string(), to_system.to_string())
                )
            }))
    }

    pub(crate) fn get_jump_gate(&self, start_system: &str) -> Option<String> {
        self.router.get_jump_gate(start_system)
    }
}

#[derive(Debug, Clone)]
pub struct JumpGateRouter {
    all_connections: Vec<GateConnection>,
    pub system_to_gate_mapping: HashMap<String, String>,
}

impl JumpGateRouter {
    pub fn new(
        all_connections: Vec<GateConnection>,
        system_to_gate_mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            all_connections,
            system_to_gate_mapping,
        }
    }

    pub fn find_jump_route(
        &self,
        from_system: &str,
        to_system: &str,
        no_under_construction: bool,
    ) -> Option<Vec<JumpConnection>> {
        let mut unvisited: Vec<GateConnection> = self.all_connections.clone();
        let mut to_visit: PriorityQueue<JumpConnection, Reverse<i64>> = PriorityQueue::new();
        let mut visited: HashMap<String, JumpConnection> = HashMap::new();

        // tracing::info!(from_system = %from_system, to_system = %to_system, "Finding route");

        let start_conns = Self::get_connections(from_system, &mut unvisited, no_under_construction);
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
                return Some(Self::get_route(
                    visited,
                    from_system.to_string(),
                    to_system.to_string(),
                ));
            }
            let conns =
                Self::get_connections(&conn.end_system, &mut unvisited, no_under_construction);
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

        None
    }

    fn get_connections(
        from_system: &str,
        unvisited: &mut Vec<GateConnection>,
        no_under_construction: bool,
    ) -> Vec<GateConnection> {
        let conns = unvisited
            .iter()
            .filter(|conn| {
                (conn.system_a == from_system || conn.system_b == from_system)
                    && !(no_under_construction
                        && (conn.under_construction_a || conn.under_construction_b))
            })
            .cloned()
            .collect::<Vec<_>>();
        unvisited.retain(|conn| {
            !((conn.system_a == from_system || conn.system_b == from_system)
                && !(no_under_construction
                    && (conn.under_construction_a || conn.under_construction_b)))
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

    pub(crate) fn get_jump_gate(&self, start_system: &str) -> Option<String> {
        self.system_to_gate_mapping.get(start_system).cloned()
    }
}

pub async fn generate_all_connections(
    database_pool: &database::DbPool,
) -> Result<(Vec<GateConnection>, HashMap<String, String>)> {
    let all_connections =
        database::JumpGateConnection::get_all(database_pool, database::PaginatedQuery::unpaged())
            .await?
            .items;

    let mut connection_map: HashMap<(String, String), GateConnection> = HashMap::new();

    for connection in all_connections {
        let mut pair = [connection.from.clone(), connection.to.clone()];
        pair.sort(); // Ensure the pair is always in a consistent order
        let entry = connection_map.entry((pair[0].clone(), pair[1].clone()));

        let entry = entry.or_insert_with(|| GateConnection {
            jump_gate_waypoint_a: pair[0].clone(),
            jump_gate_waypoint_b: pair[1].clone(),
            under_construction_a: false,
            under_construction_b: false,
            from_a: false,
            from_b: false,
            system_pos_a: (0, 0),
            system_a: String::new(),
            system_pos_b: (0, 0),
            system_b: String::new(),
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

    let mut system_to_gate_mapping: HashMap<String, String> = HashMap::new();

    let mut waypoints = HashMap::new();
    for waypoint in connection_map
        .keys()
        .flat_map(|k| [k.0.clone(), k.1.clone()])
        .collect::<HashSet<_>>()
    {
        let wp = database::Waypoint::get_by_id(database_pool, &waypoint).await?;
        if let Some(wp) = wp {
            system_to_gate_mapping.insert(wp.system_symbol.clone(), waypoint.clone());
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
        .filter_map(|mut c| {
            let wp_a = waypoints.get(&c.jump_gate_waypoint_a)?;
            let wp_b = waypoints.get(&c.jump_gate_waypoint_b)?;
            let sys_a = systems.get(&wp_a.system_symbol)?;
            let sys_b = systems.get(&wp_b.system_symbol)?;

            c.system_pos_a = (sys_a.x, sys_a.y);
            c.system_pos_b = (sys_b.x, sys_b.y);
            c.system_a = wp_a.system_symbol.clone();
            c.system_b = wp_b.system_symbol.clone();

            let distance = ((c.system_pos_a.0 - c.system_pos_b.0).pow(2)
                + (c.system_pos_a.1 - c.system_pos_b.1).pow(2)) as f64;
            c.distance = distance.sqrt();
            c.under_construction_a = wp_a.is_under_construction;
            c.under_construction_b = wp_b.is_under_construction;

            Some(c)
        })
        .collect::<Vec<_>>();

    Ok((connections, system_to_gate_mapping))
}
