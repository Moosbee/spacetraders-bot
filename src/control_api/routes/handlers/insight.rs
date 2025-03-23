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
