use std::collections::HashMap;

use database::DatabaseConnector;
use log::debug;
use warp::reply::Reply;

use crate::{
    control_api::types::{Result, ServerError},
    utils::ConductorContext,
};

pub async fn handle_get_api_counter(context: ConductorContext) -> Result<impl Reply> {
    let counter = context.api.get_limiter().get_counter();
    Ok(warp::reply::json(&serde_json::json!({"counter": counter})))
}

pub async fn handle_get_running_contract_shipments(
    context: ConductorContext,
) -> Result<impl Reply> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    context
        .contract_manager
        .sender
        .send(
            crate::manager::contract_manager::ContractShipmentMessage::GetRunning { callback: tx },
        )
        .await
        .map_err(|e| ServerError::Server(format!("Failed to send message: {}", e)))?;

    let erg: Vec<database::ContractShipment> = rx
        .await
        .map_err(|e| ServerError::Server(format!("Failed to receive message: {}", e)))?
        .map_err(|e| ServerError::Server(e.to_string()))?;

    Ok(warp::reply::json(&serde_json::json!({"shipments": erg})))
}

pub async fn handle_get_running_construction_shipments(
    context: ConductorContext,
) -> Result<impl Reply> {
    let shipments = context
        .construction_manager
        .get_running_shipments()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;
    Ok(warp::reply::json(
        &serde_json::json!({"shipments": shipments}),
    ))
}

pub async fn handle_get_mining_assignments(context: ConductorContext) -> Result<impl Reply> {
    let assignments = context
        .mining_manager
        .get_assignments()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;
    Ok(warp::reply::json(
        &serde_json::json!({"assignments": assignments}),
    ))
}

pub async fn handle_get_scrapping_info(
    symbol: String,
    context: ConductorContext,
) -> Result<impl Reply> {
    let ship_clone = context
        .ship_manager
        .get_clone(&symbol)
        .ok_or(ServerError::NotFound)?;
    let info = context
        .scrapping_manager
        .get_info(ship_clone)
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;
    Ok(warp::reply::json(&serde_json::json!({"info": info})))
}

pub async fn handle_get_ships_to_purchase(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting ships to purchase");
    let all_ships = context
        .ship_manager
        .get_all_clone()
        .await
        .into_values()
        .collect::<Vec<_>>();

    let all_systems_hashmap: HashMap<String, HashMap<String, database::Waypoint>> =
        database::Waypoint::get_hash_map(&context.database_pool)
            .await
            .map_err(|e| ServerError::Server(e.to_string()))?;
    let all_connections: Vec<database::JumpGateConnection> =
        database::JumpGateConnection::get_all(&context.database_pool)
            .await
            .map_err(|e| ServerError::Server(e.to_string()))?;

    let mut connection_hash_map: HashMap<String, Vec<database::JumpGateConnection>> =
        HashMap::new();

    for connection in all_connections {
        let entry = connection_hash_map
            .entry(connection.from.clone())
            .or_default();
        entry.push(connection);
    }
    debug!("Preparation complete");
    let scrap_ships = crate::manager::scrapping_manager::ScrappingManager::get_required_ships(
        &all_ships,
        &all_systems_hashmap,
    )
    .map_err(|e| ServerError::Server(e.to_string()))?;

    debug!("Got Scrap ships: {}", scrap_ships.len());

    let markets_per_ship = { context.config.read().await.markets_per_ship };

    let trading_ships = crate::manager::trade_manager::TradeManager::get_required_ships(
        &all_ships,
        &all_systems_hashmap,
        markets_per_ship,
    )
    .map_err(|e| ServerError::Server(e.to_string()))?;

    debug!("Got Trading ships: {}", trading_ships.len());

    let mining_ships = context
        .mining_manager
        .get_ships(&context)
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    debug!("Got Mining ships: {}", mining_ships.len());

    let construction_ships = context
        .construction_manager
        .get_ships(&context)
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    debug!("Got Construction ships: {}", construction_ships.len());

    let chart_ships = crate::manager::chart_manager::ChartManager::get_required_ships(
        &all_ships,
        &all_systems_hashmap,
        &connection_hash_map,
    )
    .map_err(|e| ServerError::Server(e.to_string()))?;

    debug!("Got Chart ships: {}", chart_ships.len());

    let contract_ships = context
        .contract_manager
        .get_ships(&context)
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    debug!("Got Contract ships: {}", contract_ships.len());

    Ok(warp::reply::json(&serde_json::json!({
      "chart":chart_ships,
      "construction":construction_ships,
      "contract":contract_ships,
      "mining":mining_ships,
      "scrap": scrap_ships,
      "trading":trading_ships,
    })))
}

pub async fn handle_get_possible_trades(context: ConductorContext) -> Result<impl Reply> {
    let trades = context
        .trade_manager
        .get_trades()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;
    Ok(warp::reply::json(&serde_json::json!({"trades": trades})))
}

pub async fn handle_get_run_info(context: ConductorContext) -> Result<impl Reply> {
    let info = { context.run_info.read().await.clone() };
    Ok(warp::reply::json(&serde_json::json!(info)))
}

pub async fn handle_get_config(context: ConductorContext) -> Result<impl Reply> {
    let info = { context.config.read().await.clone() };
    Ok(warp::reply::json(&serde_json::json!(info)))
}

pub async fn handle_update_config(
    body: serde_json::Value,
    context: ConductorContext,
) -> Result<impl Reply> {
    {
        let new_config = serde_json::from_value::<crate::utils::Config>(body.clone())
            .map_err(|e| ServerError::BadRequest(e.to_string()))?;
        let mut config = context.config.write().await;
        *config = new_config;
    }

    let info = { context.config.read().await.clone() };

    let serde_string =
        serde_json::to_string(&info).map_err(|e| ServerError::Server(e.to_string()))?;
    tokio::fs::write("config.json", serde_string)
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    Ok(warp::reply::json(&serde_json::json!(info)))
}
