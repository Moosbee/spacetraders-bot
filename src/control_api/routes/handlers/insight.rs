use warp::reply::Reply;

use crate::{
    control_api::types::{Result, ServerError},
    types::ConductorContext,
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

    let erg: Vec<crate::sql::ContractShipment> = rx
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
    let scrap_ships = context
        .scrapping_manager
        .get_ships()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    let trading_ships = context
        .trade_manager
        .get_ships()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    let mining_ships = context
        .mining_manager
        .get_ships()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    let construction_ships = context
        .construction_manager
        .get_ships()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    let chart_ships = context
        .chart_manager
        .get_ships()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    let contract_ships = context
        .contract_manager
        .get_ships()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

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
