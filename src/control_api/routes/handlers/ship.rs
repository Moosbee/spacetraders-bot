use std::str::FromStr;

use database::DatabaseConnector;
use log::debug;
use utils::WaypointCan;
use warp::reply::Reply;

use crate::{
    control_api::types::{Result, ServerError},
    error,
    utils::ConductorContext,
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
    let mut sql_ship = database::ShipInfo::get_by_symbol(&context.database_pool, &symbol)
        .await
        .map_err(ServerError::Database)?
        .ok_or(ServerError::NotFound)?;

    sql_ship.active = !sql_ship.active;
    database::ShipInfo::insert(&context.database_pool, &sql_ship)
        .await
        .map_err(ServerError::Database)?;

    Ok(warp::reply::json(&sql_ship))
}

pub async fn handle_change_role(
    symbol: String,
    body: serde_json::Value,
    context: ConductorContext,
) -> Result<impl Reply> {
    let mut sql_ship = database::ShipInfo::get_by_symbol(&context.database_pool, &symbol)
        .await
        .map_err(ServerError::Database)?
        .ok_or(ServerError::NotFound)?;

    let trade_symbol_str = body["role"]
        .as_str()
        .ok_or(ServerError::BadRequest("Missing role".into()))?;
    let trade_symbol = database::ShipInfoRole::try_from(trade_symbol_str)
        .map_err(|_| ServerError::BadRequest("Invalid Role".into()))?;

    sql_ship.role = trade_symbol;
    database::ShipInfo::insert(&context.database_pool, &sql_ship)
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

    if ship.role != database::ShipInfoRole::Manuel {
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
        .purchase_ship(Some(purchase_ship_request.clone()))
        .await
        .map_err(crate::error::Error::from)
        .map_err(ServerError::from)?;

    database::Agent::insert(
        &context.database_pool,
        &database::Agent::from(*resp.data.agent),
    )
    .await
    .map_err(|err| ServerError::Server(err.to_string()))?;

    let transaction = database::ShipyardTransaction::try_from(*resp.data.transaction)
        .map_err(|err| ServerError::Server(err.to_string()))?;

    database::ShipyardTransaction::insert(&context.database_pool, &transaction)
        .await
        .map_err(|err| ServerError::Server(err.to_string()))?;

    crate::ship::MyShip::update_info_db((*resp.data.ship).clone(), &context.database_pool)
        .await
        .map_err(|err| ServerError::Server(err.to_string()))?;

    let shipyard = context
        .api
        .get_shipyard(
            &resp.data.ship.nav.system_symbol,
            &resp.data.ship.nav.waypoint_symbol,
        )
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

    crate::manager::scrapping_manager::utils::update_shipyard(
        &context.database_pool,
        *shipyard.data,
    )
    .await
    .map_err(|err| ServerError::Server(err.to_string()))?;

    context.ship_tasks.start_ship(ship_info.clone()).await;

    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "shipSymbol": ship_info.symbol,
        "transaction": transaction
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

    if ship.role != database::ShipInfoRole::Manuel {
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
        database::TransactionReason::None,
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

pub async fn handle_jump_ship(
    symbol: String,
    body: serde_json::Value,
    context: ConductorContext,
) -> crate::control_api::types::Result<impl Reply> {
    debug!("Jumping ship {}", symbol);

    let mut ship_guard = context
        .ship_manager
        .try_get_mut(&symbol)
        .await
        .ok_or_else(|| ServerError::BadRequest("Ship was locked".into()))?;

    let ship = ship_guard
        .value_mut()
        .ok_or_else(|| ServerError::BadRequest("Ship not found".into()))?;

    if ship.role != database::ShipInfoRole::Manuel {
        return Err(ServerError::BadRequest("Ship not in Manuel mode".into()).into());
    }

    let waypoint_symbol = body["waypointSymbol"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or(ServerError::BadRequest("Missing waypointSymbol".into()))?;

    ship.ensure_undocked(&context.api)
        .await
        .map_err(ServerError::from)?;

    let jump_data = ship
        .jump(&context.api, &waypoint_symbol)
        .await
        .map_err(ServerError::from)?;

    database::Agent::insert(
        &context.database_pool,
        &database::Agent::from((*jump_data.data.agent).clone()),
    )
    .await
    .map_err(ServerError::from)?;

    let transaction =
        database::MarketTransaction::try_from(jump_data.data.transaction.as_ref().clone())
            .map_err(ServerError::from)?
            .with(database::TransactionReason::None);
    database::MarketTransaction::insert(&context.database_pool, &transaction)
        .await
        .map_err(ServerError::from)?;

    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "shipSymbol": symbol,
        "waypointSymbol": waypoint_symbol,
    })))
}

pub async fn handle_warp_ship(
    symbol: String,
    body: serde_json::Value,
    context: ConductorContext,
) -> crate::control_api::types::Result<impl Reply> {
    debug!("Warping ship {}", symbol);

    let mut ship_guard = context
        .ship_manager
        .try_get_mut(&symbol)
        .await
        .ok_or_else(|| ServerError::BadRequest("Ship was locked".into()))?;

    let ship = ship_guard
        .value_mut()
        .ok_or_else(|| ServerError::BadRequest("Ship not found".into()))?;

    if ship.role != database::ShipInfoRole::Manuel {
        return Err(ServerError::BadRequest("Ship not in Manuel mode".into()).into());
    }

    let waypoint_symbol = body["waypointSymbol"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or(ServerError::BadRequest("Missing waypointSymbol".into()))?;

    ship.ensure_undocked(&context.api)
        .await
        .map_err(ServerError::from)?;

    let erg = ship
        .warp(&context.api, &waypoint_symbol)
        .await
        .map_err(ServerError::from)?;

    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "shipSymbol": symbol,
        "waypointSymbol": waypoint_symbol,
        "warp_data": erg
    })))
}

pub async fn handle_chart_waypoint(
    symbol: String,
    context: ConductorContext,
) -> crate::control_api::types::Result<impl Reply> {
    let erg = context
        .api
        .create_chart(&symbol)
        .await
        .map_err(|err| ServerError::Server(err.to_string()))?;

    let sql_waypoint = (&*erg.data.waypoint).into();

    database::Waypoint::insert(&context.database_pool, &sql_waypoint)
        .await
        .map_err(|err| ServerError::Server(err.to_string()))?;

    if sql_waypoint.is_marketplace() {
        let market = context
            .api
            .get_market(&sql_waypoint.system_symbol, &sql_waypoint.symbol)
            .await
            .map_err(|err| ServerError::Server(err.to_string()))?;

        crate::manager::scrapping_manager::utils::update_market(
            *market.data,
            &context.database_pool,
        )
        .await;
    }

    if sql_waypoint.is_shipyard() {
        let shipyard = context
            .api
            .get_shipyard(&sql_waypoint.system_symbol, &sql_waypoint.symbol)
            .await
            .map_err(|err| ServerError::Server(err.to_string()))?;

        crate::manager::scrapping_manager::utils::update_shipyard(
            &context.database_pool,
            *shipyard.data,
        )
        .await
        .map_err(|err| ServerError::Server(err.to_string()))?;
    }

    Ok(warp::reply::json(&serde_json::json!({"chart": erg})))
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

    if ship.role != database::ShipInfoRole::Manuel {
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
        .ok_or_else(|| error::Error::General("Ship not found".to_string()))?;

    ship.nav_to(
        waypoint_id,
        true,
        database::TransactionReason::None,
        context,
    )
    .await?;

    Ok(())
}
