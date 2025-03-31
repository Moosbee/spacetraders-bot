use std::collections::HashMap;

use crate::{sql, types::ConductorContext, utils::get_system_symbol};

use super::{nav_mode::NavMode, simple_pathfinding::SimplePathfinder, SimpleConnection};

pub struct Pathfinder {
    pub range: u32,
    pub nav_mode: NavMode,
    pub start_range: u32,
    pub only_markets: bool,
    pub context: ConductorContext,
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
            let system = sql::Waypoint::get_by_system(&self.context.database_pool, &start_system)
                .await?
                .into_iter()
                .map(|w| (w.symbol.clone(), w))
                .collect::<HashMap<_, _>>();
            let simple = self.get_simple(system);
            return simple.find_route_system(start_symbol, end_symbol);
        }
        todo!()
    }

    pub fn get_simple(&self, waypoints: HashMap<String, sql::Waypoint>) -> SimplePathfinder {
        SimplePathfinder {
            range: self.range,
            nav_mode: self.nav_mode,
            system: waypoints,
            start_range: self.start_range,
            only_markets: self.only_markets,
        }
    }
}
