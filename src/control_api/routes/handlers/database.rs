use log::debug;
use warp::reply::Reply;

use crate::{
    control_api::types::{Result, ServerError},
    sql::{self, DatabaseConnector},
    workers::types::ConductorContext,
};

pub async fn handle_get_trade_routes(context: ConductorContext) -> Result<impl Reply> {
    let trade_routes = sql::TradeRoute::get_summarys(&context.database_pool)
        .await
        .map_err(ServerError::Database)?;
    Ok(warp::reply::json(&trade_routes))
}

pub async fn handle_get_contract(id: String, context: ConductorContext) -> Result<impl Reply> {
    let contract = sql::Contract::get_by_id(&context.database_pool, &id)
        .await
        .map_err(ServerError::Database)?;

    let deliveries = sql::ContractDelivery::get_by_contract_id(&context.database_pool, &id)
        .await
        .map_err(ServerError::Database)?;

    let transactions = sql::MarketTransaction::get_by_reason(
        &context.database_pool,
        sql::TransactionReason::Contract(id.clone()),
    )
    .await
    .map_err(ServerError::Database)?;

    let shipments = sql::ContractShipment::get_by_contract_id(&context.database_pool, &id)
        .await
        .map_err(ServerError::Database)?;

    Ok(warp::reply::json(&(
        id,
        contract,
        deliveries,
        transactions,
        shipments,
    )))
}

pub async fn handle_get_contracts(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting contracts");
    let contracts = sql::Contract::get_all_sm(&context.database_pool)
        .await
        .map_err(ServerError::Database)?;
    debug!("Got {} contracts", contracts.len());
    Ok(warp::reply::json(&contracts))
}

pub async fn handle_get_transactions(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting transactions");
    let transactions = sql::MarketTransaction::get_all(&context.database_pool)
        .await
        .map_err(ServerError::Database)?;
    debug!("Got {} transactions", transactions.len());
    Ok(warp::reply::json(&transactions))
}
