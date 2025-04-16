use std::collections::HashMap;

use utils::get_system_symbol;

use crate::autopilot::jump_gate_nav::{self, JumpPathfinder};

use super::{SimpleConnection, nav_mode::NavMode, simple_pathfinding::SimplePathfinder};

pub struct Pathfinder {
    pub range: u32,
    pub nav_mode: NavMode,
    pub start_range: u32,
    pub only_markets: bool,
    pub can_warp: bool,
    pub database_pool: database::DbPool,
    pub api: space_traders_client::Api,
}

impl Pathfinder {
    pub(crate) async fn get_route(
        &self,
        start_symbol: &str,
        end_symbol: &str,
    ) -> crate::error::Result<Vec<SimpleConnection>> {
        let start_system = get_system_symbol(start_symbol);
        let end_system = get_system_symbol(end_symbol);
        if start_system == end_system {
            let system = database::Waypoint::get_by_system(&self.database_pool, &start_system)
                .await?
                .into_iter()
                .map(|w| (w.symbol.clone(), w))
                .collect::<HashMap<_, _>>();
            let simple = self.get_simple(system);
            return simple.find_route_system(start_symbol, end_symbol);
        } else if !self.can_warp {
            let connections = jump_gate_nav::generate_all_connections(&self.database_pool)
                .await?
                .into_iter()
                .filter(|c| !c.under_construction_a && !c.under_construction_b)
                .collect::<Vec<_>>();
            let jump_gate = JumpPathfinder::new(connections);
            let conns = jump_gate.find_route(&start_system, &end_system);
            if conns.is_empty() {
                return Err(crate::error::Error::General("No Route found".to_string()));
            }
            let mut route = vec![];
            let start = conns.first().unwrap();
            let start_end = start.conn.get_other_system(&start.end_system).0;
            let system =
                database::Waypoint::get_by_system(&self.database_pool, &start.start_system)
                    .await?
                    .into_iter()
                    .map(|w| (w.symbol.clone(), w))
                    .collect::<HashMap<_, _>>();
            let simple = self.get_simple(system);
            route.append(&mut simple.find_route_system(start_symbol, &start_end)?);

            for conn in conns.iter() {
                let start_symbol = conn.conn.get_other_system(&conn.end_system);
                let end_symbol = conn.conn.get_other_system(&conn.start_system);
                let simple = SimpleConnection {
                    start_symbol: start_symbol.0,
                    end_symbol: end_symbol.0,
                    connection_type: crate::autopilot::connection::ConnectionType::JumpGate,
                    start_is_marketplace: true,
                    end_is_marketplace: true,
                    cost: (conn.conn.distance * 1_000_000.0),
                    re_cost: (conn.conn.distance * 1_000_000.0),
                    distance: conn.conn.distance,
                };
                route.push(simple);
            }

            let end = conns.last().unwrap();
            let end_end = end.conn.get_other_system(&end.start_system).0;
            let system = database::Waypoint::get_by_system(&self.database_pool, &end.end_system)
                .await?
                .into_iter()
                .map(|w| (w.symbol.clone(), w))
                .collect::<HashMap<_, _>>();
            let simple = self.get_simple(system);
            route.append(&mut simple.find_route_system(&end_end, end_symbol)?);

            return Ok(route);
        }
        todo!()
    }

    pub fn get_simple(&self, waypoints: HashMap<String, database::Waypoint>) -> SimplePathfinder {
        SimplePathfinder {
            range: self.range,
            nav_mode: self.nav_mode,
            system: waypoints,
            start_range: self.start_range,
            only_markets: self.only_markets,
        }
    }
}
