use log::debug;
use warp::reply::Reply;

use crate::{control_api::types::Result, workers::types::ConductorContext};

mod database;
mod ship;

pub async fn handle_get_waypoints(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting waypoints");
    let waypoints: std::collections::HashMap<
        String,
        std::collections::HashMap<String, space_traders_client::models::Waypoint>,
    > = context
        .all_waypoints
        .iter()
        .map(|w| w.clone())
        .map(|f| (f.values().next().unwrap().system_symbol.clone(), f))
        .collect();

    debug!("Got {} waypoints", waypoints.len());
    Ok(warp::reply::json(&waypoints))
}

pub use database::handle_get_contract;
pub use database::handle_get_contracts;
pub use database::handle_get_trade_routes;
pub use database::handle_get_transactions;
pub use ship::handle_buy_ship;
pub use ship::handle_get_ships;
pub use ship::handle_navigate_ship;
pub use ship::handle_purchase_cargo_ship;
pub use ship::handle_toggle_orbit;
