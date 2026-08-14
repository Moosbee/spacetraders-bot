use std::collections::HashMap;

use tracing::instrument;
use utils::get_system_symbol;

use crate::autopilot::{
    ShipNavStats,
    jump_gate_router::{JumpGateRouter, JumpGateRouterCache},
    system_router::{SystemRouter, SystemRouterCache},
};

#[derive(Debug, Default)]
pub struct NavigatorCache {
    system_routers: HashMap<String, SystemRouterCache>,
    jump_gate_router: Option<JumpGateRouterCache>,
}
impl NavigatorCache {
    pub async fn preload_system_routers(
        &mut self,
        database_pool: &database::DbPool,
        system_symbol: &str,
    ) -> crate::error::Result<()> {
        get_system_router(&mut self.system_routers, system_symbol, database_pool).await?;
        Ok(())
    }

    pub async fn preload_jump_gate_router(
        &mut self,
        database_pool: &database::DbPool,
    ) -> crate::error::Result<()> {
        get_jump_gate_router(database_pool, &mut self.jump_gate_router).await?;
        Ok(())
    }

    pub async fn get_route(
        &mut self,
        database_pool: &database::DbPool,
        start_waypoint_symbol: &str,
        end_waypoint_symbol: &str,
        ship_stats: &ShipNavStats,
    ) -> crate::error::Result<Option<Vec<super::SimpleConnection>>> {
        let route = get_route(
            database_pool,
            start_waypoint_symbol,
            end_waypoint_symbol,
            ship_stats,
            &mut self.system_routers,
            &mut self.jump_gate_router,
        )
        .await?;
        Ok(route)
    }
}

#[instrument(level = "debug", skip(database_pool), err(Debug))]
pub async fn get_route(
    database_pool: &database::DbPool,
    start_waypoint_symbol: &str,
    end_waypoint_symbol: &str,
    ship_stats: &ShipNavStats,
    system_routers: &mut HashMap<String, SystemRouterCache>,
    jump_gate_router: &mut Option<JumpGateRouterCache>,
) -> crate::error::Result<Option<Vec<super::SimpleConnection>>> {
    let start_system = get_system_symbol(start_waypoint_symbol);
    let end_system = get_system_symbol(end_waypoint_symbol);
    if start_system == end_system {
        tracing::debug!(start = %start_waypoint_symbol, end = %end_waypoint_symbol, "Same system starting");
        let router = get_system_router(system_routers, &start_system, database_pool).await?;
        let route = router.find_route_system(
            start_waypoint_symbol,
            end_waypoint_symbol,
            ship_stats.into(),
        );
        Ok(route.map(|c| c.to_vec()))
    } else if ship_stats.can_warp {
        tracing::error!("TODO: jump gate router");
        todo!()
    } else {
        tracing::debug!(start = %start_waypoint_symbol, end = %end_waypoint_symbol, "Different system starting");
        let mut route = Vec::new();
        // get jump gate router
        let jump_gate_router = get_jump_gate_router(database_pool, jump_gate_router).await?;
        // get jump gate of start system
        let start_jump_gate = jump_gate_router.get_jump_gate(&start_system);
        if start_jump_gate.is_none() {
            return Ok(None);
        };
        let start_jump_gate = start_jump_gate.unwrap();
        // get jump gate of end system
        let end_jump_gate = jump_gate_router.get_jump_gate(&end_system);
        if end_jump_gate.is_none() {
            return Ok(None);
        };
        let end_jump_gate = end_jump_gate.unwrap();

        // get route from jump gate of start system to jump gate of end system
        let jump_gate_route =
            jump_gate_router.find_jump_route(&start_jump_gate, &end_jump_gate, true);
        if jump_gate_route.is_none() {
            return Ok(None);
        }
        // get route from start waypoint to jump gate of start system
        let start_system_router =
            get_system_router(system_routers, &start_system, database_pool).await?;
        let start_route = start_system_router.find_route_system(
            start_waypoint_symbol,
            &start_jump_gate,
            ship_stats.into(),
        );
        if start_route.is_none() {
            return Ok(None);
        };
        route.extend(start_route.unwrap().iter().cloned());

        route.extend(jump_gate_route.unwrap().iter().map(|conn| {
            let start_symbol = conn.conn.get_other_system(&conn.end_system);
            let end_symbol = conn.conn.get_other_system(&conn.start_system);
            super::SimpleConnection {
                start_symbol: start_symbol.0,
                end_symbol: end_symbol.0,
                connection_type: crate::autopilot::connection::ConnectionType::JumpGate,
                start_is_marketplace: true,
                end_is_marketplace: true,
                cost: (conn.conn.distance * 1_000_000.0),
                re_cost: (conn.conn.distance * 1_000_000.0),
                distance: conn.conn.distance,
            }
        }));
        // get route from jump gate of end system to end waypoint
        let end_system_router =
            get_system_router(system_routers, &end_system, database_pool).await?;
        let end_route = end_system_router.find_route_system(
            &end_jump_gate,
            end_waypoint_symbol,
            ship_stats.into(),
        );
        if end_route.is_none() {
            return Ok(None);
        }
        route.extend(end_route.unwrap().iter().cloned());

        Ok(Some(route))
    }
}

async fn get_jump_gate_router<'a>(
    database_pool: &database::DbPool,
    jump_gate_router_option: &'a mut Option<JumpGateRouterCache>,
) -> crate::error::Result<&'a mut JumpGateRouterCache> {
    if jump_gate_router_option.is_none() {
        let connections = super::jump_gate_router::generate_all_connections(database_pool).await?;
        let jump_gate_router =
            JumpGateRouterCache::new(JumpGateRouter::new(connections.0, connections.1));
        *jump_gate_router_option = Some(jump_gate_router);
    }
    Ok(jump_gate_router_option
        .as_mut()
        .ok_or("No jump gate router")?)
}

async fn get_system_router<'a>(
    system_routers: &'a mut HashMap<String, SystemRouterCache>,
    system_symbol: &str,
    database_pool: &database::DbPool,
) -> crate::error::Result<&'a mut SystemRouterCache> {
    if !system_routers.contains_key(system_symbol) {
        let waypoints = database::Waypoint::get_by_system(
            database_pool,
            system_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?;
        let system_router = SystemRouterCache::new(SystemRouter::new(
            system_symbol.to_string(),
            waypoints
                .items
                .into_iter()
                .map(|waypoint| (waypoint.symbol.clone(), waypoint))
                .collect(),
        ));
        system_routers.insert(system_symbol.to_string(), system_router);
    }

    system_routers
        .get_mut(system_symbol)
        .ok_or(crate::error::Error::General(
            "No system router found".to_string(),
        ))
}
