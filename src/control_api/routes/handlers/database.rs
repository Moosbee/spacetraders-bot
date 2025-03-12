use log::debug;
use warp::reply::Reply;

use crate::{
    control_api::types::{Result, ServerError},
    sql::{self, DatabaseConnector},
    types::{ConductorContext, WaypointCan},
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

pub async fn handle_get_waypoints(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting all waypoints");
    let waypoints = sql::Waypoint::get_all(&context.database_pool)
        .await
        .map_err(ServerError::Database)?;

    let mut waypoints_data = vec![];

    for waypoint in &waypoints {
        let trade_goods =
            sql::MarketTrade::get_last_by_waypoint(&context.database_pool, &waypoint.symbol)
                .await
                .map_err(ServerError::Database)?;

        let market_trade_goods =
            sql::MarketTradeGood::get_last_by_waypoint(&context.database_pool, &waypoint.symbol)
                .await
                .map_err(ServerError::Database)?;

        waypoints_data.push(serde_json::json!({
            "waypoint": waypoint,
            "trade_goods": trade_goods,
            "market_trade_goods": market_trade_goods,
        }));
    }

    debug!("Got {} waypoints", waypoints.len());
    Ok(warp::reply::json(&waypoints_data))
}

pub async fn handle_get_waypoint(symbol: String, context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting {} waypoint", symbol);
    let waypoint = sql::Waypoint::get_by_symbol(&context.database_pool, &symbol)
        .await
        .map_err(ServerError::Database)?
        .ok_or(ServerError::NotFound)?;

    let constructions = {
        let construction_material =
            sql::ConstructionMaterial::get_by_waypoint(&context.database_pool, &symbol)
                .await
                .map_err(ServerError::Database)?;

        if construction_material.is_empty() {
            None
        } else {
            Some(construction_material)
        }
    };

    let (market_trades, market_trade_goods, transactions, trade_good_history) = if waypoint
        .is_marketplace()
    {
        let market_trades = sql::MarketTrade::get_last_by_waypoint(&context.database_pool, &symbol)
            .await
            .map_err(ServerError::Database)?;

        let market_trade_goods =
            sql::MarketTradeGood::get_last_by_waypoint(&context.database_pool, &symbol)
                .await
                .map_err(ServerError::Database)?;

        let transactions = sql::MarketTransaction::get_by_waypoint(&context.database_pool, &symbol)
            .await
            .map_err(ServerError::Database)?;

        let trade_good_history =
            sql::MarketTradeGood::get_by_waypoint(&context.database_pool, &symbol)
                .await
                .map_err(ServerError::Database)?;

        debug!(
            "Got {} market_trades and {} market_trade_goods and {} transactions and {} trade good history",
            market_trades.len(),
            market_trade_goods.len(),
            transactions.len(),
            trade_good_history.len(),
        );

        (
            Some(market_trades),
            Some(market_trade_goods),
            Some(transactions),
            Some(trade_good_history),
        )
    } else {
        (None, None, None, None)
    };

    let (shipyard, ship_types, ships, ship_transactions) = if waypoint.is_shipyard() {
        let shipyard = sql::Shipyard::get_last_by_waypoint(&context.database_pool, &symbol)
            .await
            .map_err(ServerError::Database)?;
        let ship_types =
            sql::ShipyardShipTypes::get_last_by_waypoint(&context.database_pool, &symbol)
                .await
                .map_err(ServerError::Database)?;
        let ships = sql::ShipyardShip::get_last_by_waypoint(&context.database_pool, &symbol)
            .await
            .map_err(ServerError::Database)?;
        let ship_transactions =
            sql::ShipyardTransaction::get_by_waypoint(&context.database_pool, &symbol)
                .await
                .map_err(ServerError::Database)?;

        debug!(
            "Got {:?} shipyard, {} ship_types, {} ships and {} ship_transactions",
            shipyard,
            ship_types.len(),
            ships.len(),
            ship_transactions.len()
        );

        (
            Some(shipyard),
            Some(ship_types),
            Some(ships),
            Some(ship_transactions),
        )
    } else {
        (None, None, None, None)
    };

    Ok(warp::reply::json(&serde_json::json!({
        "waypoint":waypoint,
        "constructions":constructions,
        "market_trades":market_trades,
        "market_trade_goods":market_trade_goods,
        "transactions":transactions,
        "trade_good_history":trade_good_history,
        "shipyard":shipyard,
        "ship_types":ship_types,
        "ships":ships,
        "ship_transactions":ship_transactions
    })))
}

pub async fn handle_get_construction_materials(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting all construction materials");

    let construction_materials = sql::ConstructionMaterial::get_summary(&context.database_pool)
        .await
        .map_err(ServerError::Database)?;

    Ok(warp::reply::json(&construction_materials))
}

pub async fn handle_get_construction_shipments(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting all construction shipments");

    let construction_shipments = sql::ConstructionShipment::get_summary(&context.database_pool)
        .await
        .map_err(ServerError::Database)?;

    Ok(warp::reply::json(&construction_shipments))
}

pub async fn handle_get_systems(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting all systems");
    let systems = sql::RespSystem::get_all(&context.database_pool)
        .await
        .map_err(ServerError::Database)?;

    debug!("Got {} systems", systems.len());
    Ok(warp::reply::json(&systems))
}

pub async fn handle_get_system(symbol: String, context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting all systems");
    let system = sql::System::get_by_id(&context.database_pool, &symbol)
        .await
        .map_err(ServerError::Database)?
        .ok_or(ServerError::NotFound)?;

    let waypoints = sql::Waypoint::get_by_system(&context.database_pool, &symbol)
        .await
        .map_err(ServerError::Database)?;

    let mut waypoints_data = vec![];

    for waypoint in &waypoints {
        let trade_goods =
            sql::MarketTradeGood::get_last_by_waypoint(&context.database_pool, &waypoint.symbol)
                .await
                .map_err(ServerError::Database)?;

        waypoints_data.push(serde_json::json!({
            "waypoint": waypoint,
            "trade_goods": trade_goods,
        }));
    }

    debug!("Got {} waypoints", waypoints_data.len());
    Ok(warp::reply::json(&serde_json::json!({
        "system": system,
        "waypoints":waypoints_data
    })))
}

pub async fn handle_get_agents(context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting all agents");
    let agents = sql::Agent::get_last(&context.database_pool)
        .await
        .map_err(ServerError::Database)?;

    debug!("Got {} agents", agents.len());
    Ok(warp::reply::json(&agents))
}

pub async fn handle_get_agent(callsign: String, context: ConductorContext) -> Result<impl Reply> {
    debug!("Getting {} agent", callsign);
    let agents = sql::Agent::get_last_by_symbol(&context.database_pool, &callsign)
        .await
        .map_err(ServerError::Database)?
        .ok_or(ServerError::NotFound)?;

    Ok(warp::reply::json(&agents))
}

pub async fn handle_get_agent_history(
    callsign: String,
    context: ConductorContext,
) -> Result<impl Reply> {
    debug!("Getting {} agent", callsign);
    let agents = sql::Agent::get_by_symbol(&context.database_pool, &callsign)
        .await
        .map_err(ServerError::Database)?;
    debug!("Got {} agents", agents.len());

    Ok(warp::reply::json(&agents))
}
