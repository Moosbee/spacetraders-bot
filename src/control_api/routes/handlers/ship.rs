use std::str::FromStr;

use log::debug;
use warp::reply::Reply;

use crate::{
    control_api::types::{Result, ServerError},
    error,
    sql::{self, DatabaseConnector},
    types::ConductorContext,
};

pub async fn handle_get_ships(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting ships");
    let ships = context.ship_manager.get_all_clone().await;
    debug!("Got {} ships", ships.len());
    Ok(warp::reply::json(&ships))
}

pub async fn handle_toggle_activation(
    symbol: String,
    context: ConductorContext,
) -> Result<impl Reply> {
    let mut sql_ship = sql::ShipInfo::get_by_symbol(&context.database_pool, &symbol)
        .await
        .map_err(ServerError::Database)?
        .ok_or(ServerError::NotFound)?;

    sql_ship.active = !sql_ship.active;
    sql::ShipInfo::insert(&context.database_pool, &sql_ship)
        .await
        .map_err(ServerError::Database)?;

    Ok(warp::reply::json(&sql_ship))
}

pub async fn handle_change_role(
    symbol: String,
    body: serde_json::Value,
    context: ConductorContext,
) -> Result<impl Reply> {
    let mut sql_ship = sql::ShipInfo::get_by_symbol(&context.database_pool, &symbol)
        .await
        .map_err(ServerError::Database)?
        .ok_or(ServerError::NotFound)?;

    let trade_symbol_str = body["role"]
        .as_str()
        .ok_or(ServerError::BadRequest("Missing role".into()))?;
    let trade_symbol = sql::ShipInfoRole::try_from(trade_symbol_str)
        .map_err(|_| ServerError::BadRequest("Invalid Role".into()))?;

    sql_ship.role = trade_symbol;
    sql::ShipInfo::insert(&context.database_pool, &sql_ship)
        .await
        .map_err(ServerError::Database)?;

    Ok(warp::reply::json(&sql_ship))
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

    if ship.role != sql::ShipInfoRole::Manuel {
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

pub async fn handle_buy_ship(
    body: serde_json::Value,
    context: ConductorContext,
) -> crate::control_api::types::Result<impl Reply> {
    let ship_type = body["shipType"]
        .as_str()
        .ok_or(ServerError::BadRequest("Missing shipType".into()))?;

    let waypoint_symbol = body["waypointSymbol"]
        .as_str()
        .ok_or(ServerError::BadRequest("Missing waypointSymbol".into()))?;

    let purchase_ship_request = space_traders_client::models::PurchaseShipRequest::new(
        space_traders_client::models::ShipType::from_str(ship_type)
            .map_err(|_| ServerError::BadRequest("Invalid shipType".into()))?,
        waypoint_symbol.to_string(),
    );

    let resp = context
        .api
        .purchase_ship(Some(purchase_ship_request))
        .await
        .map_err(|err| ServerError::Server(err.to_string()))?;

    sql::Agent::insert(&context.database_pool, &sql::Agent::from(*resp.data.agent))
        .await
        .map_err(|err| ServerError::Server(err.to_string()))?;

    let transaction = sql::ShipyardTransaction::try_from(*resp.data.transaction)
        .map_err(|err| ServerError::Server(err.to_string()))?;

    sql::ShipyardTransaction::insert(&context.database_pool, &transaction)
        .await
        .map_err(|err| ServerError::Server(err.to_string()))?;

    let mut ship_i =
        crate::ship::MyShip::from_ship(*resp.data.ship, context.ship_manager.get_broadcaster());

    let ship_info = ship_i
        .apply_from_db(context.database_pool.clone())
        .await
        .map_err(|err| ServerError::Server(err.to_string()))?;

    crate::ship::ShipManager::add_ship(&context.ship_manager, ship_i).await;

    {
        let mut ship_g = context.ship_manager.get_mut(&ship_info.symbol).await;
        let ship = ship_g
            .value_mut()
            .ok_or_else(|| ServerError::BadRequest("Ship not found".into()))?;
        ship.notify().await;
    }

    context.ship_tasks.start_ship(ship_info).await;

    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
    })))
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

    if ship.role != sql::ShipInfoRole::Manuel {
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

    if ship.role != sql::ShipInfoRole::Manuel {
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
