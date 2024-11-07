use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::{Error, Ok};
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
        info!("Contract contract_que: {:?}", contract);
        do_contract(
            all_waypoints
                .get(&get_system_symbol(&contract))
                .unwrap()
                .clone(),
            my_ships.clone(),
            api.clone(),
            database_pool.clone(),
            contract,
            contract_ships[0].clone(),
        )
        .await?;
    }

    for _i in 0..5 {
        info!("Contract loop: {}", _i);
        let next_contract =
            negotiate_next_contract(my_ships.clone(), api.clone(), contract_ships.clone()).await?;
        info!("Next contract: {:?}", next_contract);
        do_contract(
            all_waypoints
                .get(&get_system_symbol(&next_contract))
                .unwrap()
                .clone(),
            my_ships.clone(),
            api.clone(),
            database_pool.clone(),
            next_contract,
            contract_ships[0].clone(),
        )
        .await?;
    }

    info!("Contract workers done");
    Ok(())
}

async fn negotiate_next_contract(
    my_ships: Arc<dashmap::DashMap<String, my_ship::MyShip>>,
    api: api::Api,
    contract_ships: Vec<&String>,
) -> Result<models::Contract, Error> {
    for ship in contract_ships.iter() {
        let mut current_ship = my_ships.get_mut(*ship).unwrap();
        let current_nav = current_ship.get_nav_status();
        if current_nav != models::ShipNavStatus::InTransit {
            if current_nav == models::ShipNavStatus::InOrbit {
                current_ship.dock(&api).await?;
            }

            let next_contract = api.negotiate_contract(contract_ships[0]).await?;
            return Ok(*next_contract.data.contract);
        }
    }

    Err(Error::msg(
        "No ships available to negotiate contracts. Could not negotiate next contract",
    ))
}

async fn do_contract(
    waypoints: HashMap<String, models::waypoint::Waypoint>,
    my_ships: Arc<dashmap::DashMap<String, my_ship::MyShip>>,
    api: api::Api,
    database_pool: sqlx::PgPool,
    contract: models::Contract,
    ship_symbol: String,
) -> anyhow::Result<()> {
    info!("Contract: {:?}", contract);
    if is_possible_contract(&contract, database_pool.clone()).await && is_in_deadline(&contract) {
        info!("Contract is possible");
        let contract = if !contract.accepted {
            let accept_data = api.accept_contract(&contract.id).await?;

            *accept_data.data.contract
        } else {
            contract
        };

        let finished_contract = do_trade_contract(
            contract.clone(),
            ship_symbol,
            api.clone(),
            my_ships.clone(),
            database_pool.clone(),
            waypoints.values().cloned().collect(),
        )
        .await?;

        if can_fulfill_trade(&finished_contract) {
            let _fulfilled_data = api.fulfill_contract(&finished_contract.id).await?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Contract could not be fulfilled"))
        }
    } else {
        Err(anyhow::anyhow!("Contract is not possible"))
    }
}

fn get_system_symbol(contract: &models::Contract) -> String {
    let waypoint_symbol = &contract.terms.deliver.clone().unwrap()[0].destination_symbol;
    api::Api::system_symbol(waypoint_symbol)
}

/// Does the actual contract trade using one ship
async fn do_trade_contract(
    contract: models::Contract,
    ship_symbol: String,
    api: api::Api,
    my_ships: Arc<dashmap::DashMap<String, my_ship::MyShip>>,
    database_pool: sqlx::PgPool,
    waypoints: Vec<models::Waypoint>,
) -> Result<models::Contract, Error> {
    let procurements = contract.terms.deliver.clone().unwrap();

    let mut current_contract = contract.clone();
    let mut ship = my_ships.get_mut(&ship_symbol).unwrap();

    info!("Contract: {:?}", current_contract);

    let wayps: HashMap<String, models::Waypoint> = waypoints
        .clone()
        .iter()
        .map(|w| (w.symbol.clone(), w.clone()))
        .collect();

    for procurement in procurements {
        let trade_symbol = models::TradeSymbol::from_str(&procurement.trade_symbol)?;
        let buy_waypoint_symbol =
            get_purchase_waypoint(procurement.clone(), database_pool.clone()).await?;

        let mut current_procurement = procurement.clone();

        info!("Procurement: {:?}", current_procurement);

        while current_procurement.units_fulfilled < current_procurement.units_required {
            info!(
                "Fulfilled: {}, Required: {} going to {}",
                current_procurement.units_fulfilled,
                current_procurement.units_required,
                buy_waypoint_symbol
            );

            let cargo_in_ship = ship
                .cargo
                .iter()
                .find(|f| f.0 == trade_symbol)
                .map(|f| f.1)
                .unwrap_or(0);
            if cargo_in_ship == 0 {
                ship.nav_to(
                    &buy_waypoint_symbol,
                    true,
                    &wayps,
                    &api,
                    database_pool.clone(),
                )
                .await?;

                info!("Arrived on waypoint {}", buy_waypoint_symbol);
                ship.dock(&api).await?;

                let still_required_cargo =
                    current_procurement.units_required - current_procurement.units_fulfilled;

                let purchase_volume =
                    (ship.cargo_capacity - ship.cargo_units).min(still_required_cargo);
                ship.purchase_cargo(&api, trade_symbol, purchase_volume, &database_pool)
                    .await?;

                info!(
                    "Purchased {} {} going to {}",
                    purchase_volume, trade_symbol, current_procurement.destination_symbol
                );
            } else {
                info!(
                    "Ship already has {} {} going to {}",
                    trade_symbol, cargo_in_ship, current_procurement.destination_symbol
                );
            }

            ship.nav_to(
                &current_procurement.destination_symbol,
                true,
                &wayps,
                &api,
                database_pool.clone(),
            )
            .await
            .unwrap();

            ship.dock(&api).await?;

            info!(
                "Arrived on waypoint {}",
                current_procurement.destination_symbol
            );

            let cargo_in_ship = ship
                .cargo
                .iter()
                .find(|f| f.0 == trade_symbol)
                .map(|f| f.1)
                .unwrap_or(0);

            let delivery_result = ship
                .deliver_contract(&contract.id, trade_symbol, cargo_in_ship, &api)
                .await
                .unwrap();

            current_contract = *delivery_result.data.contract.clone();

            let new_current_procurement = current_contract
                .terms
                .deliver
                .clone()
                .unwrap()
                .iter()
                .find(|f| {
                    f.trade_symbol == current_procurement.trade_symbol
                        && f.destination_symbol == current_procurement.destination_symbol
                })
                .unwrap()
                .clone();

            info!("New procurement: {:?}", new_current_procurement);
            current_procurement = new_current_procurement;
        }
    }

    Ok(current_contract)
}

fn can_fulfill_trade(contract: &models::Contract) -> bool {
    contract
        .terms
        .deliver
        .clone()
        .is_some_and(|f| f.iter().all(|d| d.units_fulfilled >= d.units_required))
}

async fn get_purchase_waypoint(
    procurement: models::ContractDeliverGood,
    database_pool: sqlx::PgPool,
) -> Result<String, Error> {
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

    Ok(buy_waypoint_symbol)
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

// fn can_still_be_accepted(contract: &models::Contract) -> bool {
//     let deadline = chrono::DateTime::parse_from_rfc3339(&contract.terms.deadline);

//     if deadline.is_err() {
//         return false;
//     }
//     let deadline = deadline.unwrap();

//     let now = chrono::Utc::now();
//     now < deadline
// }

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
