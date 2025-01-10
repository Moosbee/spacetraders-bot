use std::str::FromStr;

use log::debug;
use warp::reply::Reply;

use crate::{
    control_api::types::{Result, ServerError},
    error, ship, sql,
    workers::types::ConductorContext,
};

pub async fn handle_get_ships(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting ships");
    let ships = context.ship_manager.get_all_clone().await;
    debug!("Got {} ships", ships.len());
    Ok(warp::reply::json(&ships))
}

pub async fn handle_toggle_orbit(symbol: String, context: ConductorContext) -> Result<impl Reply> {
    let mut ship_guard = context
        .ship_manager
        .try_get_mut(&symbol)
        .await
        .ok_or_else(|| ServerError::BadRequest("Ship was locked".into()))?;

    let ship = ship_guard
        .value_mut()
        .ok_or_else(|| ServerError::BadRequest("Ship not found".into()))?;

    if ship.role != ship::Role::Manuel {
        return Err(ServerError::BadRequest("Ship not in Manuel mode".into()).into());
    }

    match ship.nav.status {
        space_traders_client::models::ShipNavStatus::InTransit => {
            if ship.nav.is_in_transit() {
                return Err(ServerError::BadRequest("Ship is in transit".into()).into());
            }
            ship.ensure_docked(&context.api).await.map_err(|err| {
                let n_err: ServerError = err.into();
                n_err
            })?;
        }
        space_traders_client::models::ShipNavStatus::InOrbit => {
            ship.ensure_docked(&context.api).await.map_err(|err| {
                let n_err: ServerError = err.into();
                n_err
            })?;
        }
        space_traders_client::models::ShipNavStatus::Docked => {
            ship.ensure_undocked(&context.api).await.map_err(|err| {
                let n_err: ServerError = err.into();
                n_err
            })?;
        }
    }

    Ok(warp::reply::json(&ship))
}

pub async fn handle_purchase_cargo_ship(
    symbol: String,
    body: serde_json::Value,
    context: ConductorContext,
) -> crate::control_api::types::Result<impl Reply> {
    let mut ship_guard = context
        .ship_manager
        .try_get_mut(&symbol)
        .await
        .ok_or_else(|| ServerError::BadRequest("Ship was locked".into()))?;

    let ship = ship_guard
        .value_mut()
        .ok_or_else(|| ServerError::BadRequest("Ship not found".into()))?;

    if ship.role != ship::Role::Manuel {
        return Err(ServerError::BadRequest("Ship not in Manuel mode".into()).into());
    }

    let trade_symbol_str = body["tradeSymbol"]
        .as_str()
        .ok_or(ServerError::BadRequest("Missing tradeSymbol".into()))?;
    let trade_symbol = space_traders_client::models::TradeSymbol::from_str(trade_symbol_str)
        .map_err(|_| ServerError::BadRequest("Invalid tradeSymbol".into()))?;

    let units: i32 = body["units"]
        .as_i64()
        .ok_or(ServerError::BadRequest("Missing units".into()))?
        .try_into()
        .map_err(|_| ServerError::BadRequest("Invalid units".into()))?;

    ship.ensure_docked(&context.api).await.map_err(|err| {
        let n_err: ServerError = err.into();
        n_err
    })?;

    ship.purchase_cargo(
        &context.api,
        &trade_symbol,
        units,
        &context.database_pool,
        sql::TransactionReason::None,
    )
    .await
    .map_err(|err| ServerError::Server(err.to_string()))?;

    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "shipSymbol": symbol,
        "tradeSymbol": trade_symbol,
        "units": units
    })))
}

pub async fn handle_navigate_ship(
    symbol: String,
    body: serde_json::Value,
    context: ConductorContext,
) -> crate::control_api::types::Result<impl Reply> {
    let mut ship_guard = context
        .ship_manager
        .try_get_mut(&symbol)
        .await
        .ok_or_else(|| ServerError::BadRequest("Ship was locked".into()))?;

    let ship = ship_guard
        .value_mut()
        .ok_or_else(|| ServerError::BadRequest("Ship not found".into()))?;

    if ship.role != ship::Role::Manuel {
        return Err(ServerError::BadRequest("Ship not in Manuel mode".into()).into());
    }

    let waypoint_id = body["waypointSymbol"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or(ServerError::BadRequest("Missing waypointSymbol".into()))?;
    {
        let symbol = symbol.clone();
        let waypoint_id = waypoint_id.clone();
        let context = context.clone();

        tokio::spawn(async move {
            if let Err(e) = navigate_ship(&context, &symbol, &waypoint_id).await {
                log::error!("Navigation failed: {}", e);
            }
        });
    }

    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "shipSymbol": symbol,
        "waypointSymbol": waypoint_id,
    })))
}

async fn navigate_ship(
    context: &ConductorContext,
    symbol: &str,
    waypoint_id: &str,
) -> error::Result<()> {
    let mut ship_guard = context.ship_manager.get_mut(symbol).await;
    let ship = ship_guard
        .value_mut()
        .ok_or_else(|| error::Error::General(format!("Ship not found")))?;

    let waypoints = context
        .all_waypoints
        .get(&ship.nav.system_symbol)
        .ok_or_else(|| error::Error::General(format!("Waypoints not found")))?
        .clone();

    ship.nav_to(
        waypoint_id,
        true,
        &waypoints,
        &context.api,
        context.database_pool.clone(),
        sql::TransactionReason::None,
    )
    .await
}
