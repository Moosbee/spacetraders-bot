
use database::DatabaseConnector;
use tracing::debug;
use tracing::instrument;
use warp::reply::Reply;

use crate::{
    control_api::types::{Result, ServerError},
    utils::ConductorContext,
};

#[instrument(skip(context))]
pub async fn handle_get_api_counter(context: ConductorContext) -> Result<impl Reply> {
    let counter = context.api.get_limiter().get_counter();
    Ok(warp::reply::json(&serde_json::json!({"counter": counter})))
}

#[instrument(skip(context))]
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

#[instrument(skip(context))]
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

#[instrument(skip(context))]
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

#[instrument(skip(context))]
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

#[instrument(skip(context))]
pub async fn handle_get_ships_to_purchase(context: ConductorContext) -> Result<impl Reply> {
    Ok(warp::reply::json(
        &serde_json::json!({"ships": "not implemented yet"}),
    ))
}

#[instrument(skip(context))]
pub async fn handle_get_possible_trades(context: ConductorContext) -> Result<impl Reply> {
    let trades = context
        .trade_manager
        .get_trades()
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;
    Ok(warp::reply::json(&serde_json::json!({"trades": trades})))
}

#[instrument(skip(context))]
pub async fn handle_get_run_info(context: ConductorContext) -> Result<impl Reply> {
    let info = { context.run_info.read().await.clone() };
    Ok(warp::reply::json(&serde_json::json!(info)))
}

#[instrument(skip(context))]
pub async fn handle_get_config(context: ConductorContext) -> Result<impl Reply> {
    let info = { context.config.read().await.clone() };
    Ok(warp::reply::json(&serde_json::json!(info)))
}

#[instrument(skip(context))]
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

    let old_string = tokio::fs::read_to_string("config.toml")
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    let info = { context.config.read().await.clone() };

    let mut toml_edit_doc = old_string
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| ServerError::Server(e.to_string()))?;

    let config_doc: toml_edit::DocumentMut =
        toml_edit::ser::to_document(&info).map_err(|e| ServerError::Server(e.to_string()))?;

    toml_edit_doc.extend(config_doc.iter());

    let toml_string = toml_edit_doc.to_string();
    tokio::fs::write("config.toml", toml_string)
        .await
        .map_err(|e| ServerError::Server(e.to_string()))?;

    Ok(warp::reply::json(&serde_json::json!(info)))
}

#[instrument(skip(context))]
pub async fn handle_get_budget_info(context: ConductorContext) -> Result<impl Reply> {
    let budget_info: crate::manager::budget_manager::BudgetInfo =
        context.budget_manager.get_budget_info().await;
    let all_reservations: Vec<database::ReservedFund> =
        database::ReservedFund::get_all(&context.database_pool)
            .await
            .map_err(|e| ServerError::Server(e.to_string()))?;
    debug!("All Reservations from DB: {:?}", all_reservations);
    Ok(warp::reply::json(&serde_json::json!({
        "budget_info": budget_info,
        "all_reservations": all_reservations,
    })))
}
