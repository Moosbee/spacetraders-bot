use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::Error;
use log::{debug, info};
use space_traders_client::models::{self, market_trade_good};
use tokio::time::sleep;

use crate::{api, my_ship, sql};
pub async fn contract_conductor(
    api: api::Api,
    database_pool: sqlx::PgPool,
    ship_roles: std::collections::HashMap<String, my_ship::Role>,
    my_ships: Arc<dashmap::DashMap<String, my_ship::MyShip>>,
    all_waypoints: Arc<dashmap::DashMap<String, HashMap<String, models::waypoint::Waypoint>>>,
) -> Result<(), Error> {
    info!("Starting contract workers");
    sleep(Duration::from_millis(500)).await;

    let contract_ships = ship_roles
        .iter()
        .filter(|ship| ship.1 == &my_ship::Role::Contract)
        .map(|ship| ship.0)
        .collect::<Vec<_>>();

    if contract_ships.len() == 0 {
        return Err(Error::msg("No ships assigned to me"));
    }

    let contracts = api.get_all_contracts(20).await?;

    let mut contract_que: VecDeque<models::Contract> = VecDeque::from(
        contracts
            .iter()
            .filter(|f| !f.fulfilled)
            .map(|f| f.clone())
            .collect::<Vec<models::Contract>>(),
    );

    while let Some(contract) = contract_que.pop_front() {
        info!("Contract: {:?}", contract);
        if is_possible_contract(&contract, database_pool.clone()).await && is_in_deadline(&contract)
        {
            info!("Contract is possible");

            let destination_symbol_split = &contract.terms.deliver.as_ref().unwrap()[0]
                .destination_symbol
                .split("-")
                .collect::<Vec<&str>>();

            let system_symbol = format!(
                "{}-{}",
                destination_symbol_split[0], destination_symbol_split[1]
            );

            do_contract_trade_contract(
                contract.clone(),
                contract_ships[0].clone(),
                api.clone(),
                my_ships.clone(),
                database_pool.clone(),
                all_waypoints
                    .get(&system_symbol)
                    .unwrap()
                    .values()
                    .cloned()
                    .collect(),
            )
            .await?;
        } else {
            info!("Contract is not possible");
        }
    }

    info!("Contract workers done");
    Ok(())
}

async fn do_contract_trade_contract(
    contract: models::Contract,
    ship_symbol: String,
    api: api::Api,
    my_ships: Arc<dashmap::DashMap<String, my_ship::MyShip>>,
    database_pool: sqlx::PgPool,
    waypoints: Vec<models::Waypoint>,
) -> Result<(), Error> {
    let procurements = contract.terms.deliver.clone().unwrap();

    let ship = my_ships.get(&ship_symbol).unwrap();
    info!("Ship i i {:?}", ship);

    let djirk = ship.get_dijkstra(waypoints.clone());
    info!("Ship dijkstra: {:?}", djirk);

    for procurement in procurements {
        let trade_symbol = models::TradeSymbol::from_str(&procurement.trade_symbol)?;
        let market_trades = sql::get_last_market_trades_symbol(&database_pool, &trade_symbol).await;
        let market_trade_goods = sql::get_last_trade_markets(&database_pool, &trade_symbol).await;

        let buy_waypoint_symbol = if market_trades.len() == market_trade_goods.len() {
            let mut market_trade_goods = market_trade_goods
                .iter()
                .filter(|f| !(f.r#type == market_trade_good::Type::Export))
                .collect::<Vec<_>>();

            market_trade_goods.sort_by(|a, b| a.purchase_price.cmp(&b.purchase_price));

            let market_trade_good = market_trade_goods.first().unwrap();
            debug!(
                "Market Trade Goods: {:?}, Market Trade Good: {:?}",
                market_trade_goods, market_trade_good
            );

            market_trade_good.waypoint_symbol.clone()
        } else {
            let market_trades = market_trades
                .iter()
                .filter(|f| !(f.r#type == market_trade_good::Type::Export))
                .collect::<Vec<_>>();

            let market_trade_good = market_trades.first().unwrap();

            debug!(
                "Market Trades: {:?}, Market Trade Good: {:?}",
                market_trades, market_trade_good
            );
            market_trade_good.waypoint_symbol.clone()
        };
    }

    Ok(())
}

fn is_in_deadline(contract: &models::Contract) -> bool {
    let deadline = chrono::DateTime::parse_from_rfc3339(&contract.terms.deadline);

    if deadline.is_err() {
        return false;
    }
    let deadline = deadline.unwrap();

    let now = chrono::Utc::now();
    now < deadline
}

async fn is_possible_contract(contract: &models::Contract, database_pool: sqlx::PgPool) -> bool {
    match contract.r#type {
        models::contract::Type::Procurement => {
            if contract.terms.deliver.is_none() {
                return false;
            }

            let contract_deliver = contract.terms.deliver.clone().unwrap();

            let market_trade_goods = sql::get_last_market_trades(&database_pool).await;

            for req_item in contract_deliver {
                if req_item.units_required - req_item.units_fulfilled <= 0 {
                    continue;
                }

                let symbol = models::TradeSymbol::from_str(&req_item.trade_symbol);
                if symbol.is_err() {
                    return false;
                }
                let symbol = symbol.unwrap();

                if !market_trade_goods.iter().any(|f| {
                    f.symbol == symbol
                        && (f.r#type == market_trade_good::Type::Export
                            || f.r#type == market_trade_good::Type::Exchange)
                }) {
                    return false;
                }
            }
            true
        }
        models::contract::Type::Shuttle => false,
        models::contract::Type::Transport => false,
    }
}

// fn get_next_contract(contracts: Vec<Contract>) -> Option<Contract> {
//     let mut unfinished_contracts = contracts
//         .iter()
//         .filter(|cont| !cont.fulfilled && cont.accepted)
//         .map(|f| f.clone())
//         .collect::<Vec<_>>();

//     let mut unaccepted_contracts = contracts
//         .iter()
//         .filter(|cont| !cont.fulfilled && !cont.accepted)
//         .map(|f| f.clone())
//         .collect::<Vec<_>>();

//     unfinished_contracts.append(&mut unaccepted_contracts);

//     unfinished_contracts.first().cloned()
// }
