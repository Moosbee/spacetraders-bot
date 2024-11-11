use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Error, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use log::{debug, info};
use space_traders_client::models::{self, Contract, ContractDeliverGood, TradeSymbol, Waypoint};
use tokio::time::sleep;

use crate::{
    api::Api,
    my_ship::{MyShip, Role},
    sql,
};

// Constants
const SLEEP_DURATION: u64 = 500;
const MAX_CONTRACTS: i32 = 20;
const MAX_CONTRACT_ATTEMPTS: u32 = 5;

/// Main contract conductor that manages contract operations
pub async fn contract_conductor(
    api: Api,
    database_pool: sqlx::PgPool,
    ship_roles: HashMap<String, Role>,
    my_ships: Arc<DashMap<String, MyShip>>,
    all_waypoints: Arc<DashMap<String, HashMap<String, Waypoint>>>,
) -> Result<()> {
    info!("Starting contract workers");
    sleep(Duration::from_millis(SLEEP_DURATION)).await;

    let contract_ships = get_contract_ships(&ship_roles)?;
    let primary_ship = &contract_ships[0];

    // Process existing contracts
    let mut contract_queue = get_unfulfilled_contracts(&api).await?;
    while let Some(contract) = contract_queue.pop_front() {
        process_contract(
            &contract,
            primary_ship,
            &api,
            &database_pool,
            &my_ships,
            &all_waypoints,
        )
        .await?;
    }

    // Process new contracts
    for i in 0..MAX_CONTRACT_ATTEMPTS {
        info!("Contract loop: {}", i);
        let next_contract =
            negotiate_next_contract(my_ships.clone(), &api, &contract_ships).await?;
        process_contract(
            &next_contract,
            primary_ship,
            &api,
            &database_pool,
            &my_ships,
            &all_waypoints,
        )
        .await?;
    }

    info!("Contract workers done");
    Ok(())
}

/// Get ships assigned to contract role
fn get_contract_ships(ship_roles: &HashMap<String, Role>) -> Result<Vec<String>> {
    let ships: Vec<String> = ship_roles
        .iter()
        .filter(|(_, role)| **role == Role::Contract)
        .map(|(symbol, _)| symbol.clone())
        .collect();

    if ships.is_empty() {
        return Err(Error::msg("No ships assigned to contract role"));
    }
    Ok(ships)
}

/// Get all unfulfilled contracts
async fn get_unfulfilled_contracts(api: &Api) -> Result<VecDeque<Contract>> {
    let contracts = api.get_all_contracts(MAX_CONTRACTS).await?;
    Ok(VecDeque::from(
        contracts
            .into_iter()
            .filter(|c| !c.fulfilled)
            .collect::<Vec<_>>(),
    ))
}

/// Process a single contract
async fn process_contract(
    contract: &Contract,
    ship_symbol: &str,
    api: &Api,
    database_pool: &sqlx::PgPool,
    my_ships: &Arc<DashMap<String, MyShip>>,
    all_waypoints: &Arc<DashMap<String, HashMap<String, Waypoint>>>,
) -> Result<()> {
    info!("Processing contract: {:?}", contract);

    if !is_contract_viable(contract, database_pool).await? {
        return Err(Error::msg("Contract is not viable"));
    }

    let contract = if !contract.accepted {
        info!("Accepting contract: {}", contract.id);
        let accept_data = api.accept_contract(&contract.id).await?;
        accept_data.data.contract.as_ref().clone()
    } else {
        contract.clone()
    };

    let waypoints = all_waypoints
        .get(&get_system_symbol(&contract))
        .context("System not found")?
        .clone();

    sql::update_contract(database_pool, &contract).await;

    let finished_contract = execute_trade_contract(
        contract.clone(),
        ship_symbol.to_string(),
        api,
        my_ships,
        database_pool,
        waypoints.values().cloned().collect(),
    )
    .await
    .unwrap();

    if can_fulfill_trade(&finished_contract) {
        let fulfill_contract_data = api.fulfill_contract(&finished_contract.id).await?;
        sql::update_contract(database_pool, &fulfill_contract_data.data.contract).await;

        Ok(())
    } else {
        Err(Error::msg("Contract could not be fulfilled"))
    }
}

/// Execute the actual trading for a contract
async fn execute_trade_contract(
    contract: Contract,
    ship_symbol: String,
    api: &Api,
    my_ships: &Arc<DashMap<String, MyShip>>,
    database_pool: &sqlx::PgPool,
    waypoints: Vec<Waypoint>,
) -> Result<Contract> {
    let procurements = contract
        .terms
        .deliver
        .clone()
        .context("No delivery terms")?;
    let mut current_contract = contract;
    let waypoints_map: HashMap<_, _> = waypoints
        .iter()
        .map(|w| (w.symbol.clone(), w.clone()))
        .collect();

    let mut ship = my_ships.get_mut(&ship_symbol).context("Ship not found")?;

    for mut procurement in procurements {
        let trade_symbol = TradeSymbol::from_str(&procurement.trade_symbol)?;
        let buy_waypoint =
            get_purchase_waypoint(procurement.clone(), database_pool.clone()).await?;

        info!("Buy waypoint: {} {:?}", buy_waypoint, procurement);
        while procurement.units_fulfilled < procurement.units_required {
            current_contract = handle_procurement_cycle(
                &mut ship,
                &current_contract,
                &procurement,
                trade_symbol,
                &buy_waypoint,
                &waypoints_map,
                api,
                database_pool,
                sql::TransactionReason::Contract(current_contract.id.clone()),
            )
            .await?;

            sql::update_contract(database_pool, &current_contract).await;

            procurement = get_updated_procurement(&current_contract, &procurement)?;
        }
    }

    Ok(current_contract)
}

/// Handle a single procurement cycle (buy and deliver)
async fn handle_procurement_cycle(
    ship: &mut MyShip,
    contract: &Contract,
    procurement: &ContractDeliverGood,
    trade_symbol: TradeSymbol,
    buy_waypoint: &str,
    waypoints: &HashMap<String, Waypoint>,
    api: &Api,
    database_pool: &sqlx::PgPool,
    reason: sql::TransactionReason,
) -> Result<Contract> {
    if !ship.has_cargo(&trade_symbol) {
        ship.nav_to(
            buy_waypoint,
            true,
            waypoints,
            api,
            database_pool.clone(),
            reason.clone(),
        )
        .await
        .unwrap();
        ship.update_market(api, database_pool).await?;

        ship.dock(api).await.unwrap();

        let purchase_volume = calculate_purchase_volume(ship, procurement);
        ship.purchase_cargo(
            api,
            trade_symbol,
            purchase_volume,
            database_pool,
            reason.clone(),
        )
        .await
        .unwrap();
    }

    ship.nav_to(
        &procurement.destination_symbol,
        true,
        waypoints,
        api,
        database_pool.clone(),
        reason,
    )
    .await
    .unwrap();
    ship.dock(api).await.unwrap();

    let cargo_amount = ship.get_cargo_amount(&trade_symbol);
    let delivery_result = ship
        .deliver_contract(&contract.id, trade_symbol, cargo_amount, api)
        .await
        .unwrap();

    Ok(*delivery_result.data.contract)
}

async fn negotiate_next_contract(
    my_ships: Arc<dashmap::DashMap<String, MyShip>>,
    api: &Api,
    contract_ships: &Vec<String>,
) -> Result<models::Contract, Error> {
    for ship in contract_ships.iter() {
        let mut current_ship = my_ships.get_mut(ship).unwrap();
        let current_nav = current_ship.get_nav_status();
        if current_nav != models::ShipNavStatus::InTransit {
            if current_nav == models::ShipNavStatus::InOrbit {
                current_ship.dock(&api).await?;
            }

            let next_contract = api.negotiate_contract(&current_ship.symbol).await?;
            return Ok(*next_contract.data.contract);
        }
    }

    Err(Error::msg(
        "No ships available to negotiate contracts. Could not negotiate next contract",
    ))
}

/// Helper functions
fn get_system_symbol(contract: &Contract) -> String {
    let waypoint_symbol = &contract.terms.deliver.as_ref().unwrap()[0].destination_symbol;
    Api::system_symbol(waypoint_symbol)
}

fn calculate_purchase_volume(ship: &MyShip, procurement: &ContractDeliverGood) -> i32 {
    let remaining_required = procurement.units_required - procurement.units_fulfilled;
    (ship.cargo_capacity - ship.cargo_units).min(remaining_required)
}

fn can_fulfill_trade(contract: &Contract) -> bool {
    contract.terms.deliver.as_ref().map_or(false, |deliveries| {
        deliveries
            .iter()
            .all(|d| d.units_fulfilled >= d.units_required)
    })
}

async fn is_contract_viable(contract: &Contract, database_pool: &sqlx::PgPool) -> Result<bool> {
    if !is_in_deadline(contract) {
        return Ok(false);
    }

    match contract.r#type {
        models::contract::Type::Procurement => {
            check_procurement_viability(contract, database_pool).await
        }
        _ => Ok(false),
    }
}

fn is_in_deadline(contract: &Contract) -> bool {
    DateTime::parse_from_rfc3339(&contract.terms.deadline)
        .map(|deadline| Utc::now() < deadline)
        .unwrap_or(false)
}

async fn check_procurement_viability(
    contract: &Contract,
    database_pool: &sqlx::PgPool,
) -> Result<bool> {
    let Some(deliveries) = &contract.terms.deliver else {
        return Ok(false);
    };

    let market_trade_goods = sql::get_last_market_trades(database_pool).await;

    for delivery in deliveries {
        if delivery.units_required <= delivery.units_fulfilled {
            continue;
        }

        let symbol = TradeSymbol::from_str(&delivery.trade_symbol)?;
        if !market_trade_goods.iter().any(|trade| {
            trade.symbol == symbol
            // && matches!(trade.r#type, TradeType::Export | TradeType::Exchange)
        }) {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn get_purchase_waypoint(
    procurement: ContractDeliverGood,
    database_pool: sqlx::PgPool,
) -> Result<String> {
    let trade_symbol = TradeSymbol::from_str(&procurement.trade_symbol)?;
    let mut market_trades = sql::get_last_market_trades_symbol(&database_pool, &trade_symbol).await;
    let market_trade_goods = sql::get_last_trade_markets(&database_pool, &trade_symbol).await;

    if market_trades.len() == market_trade_goods.len() {
        let best_market = market_trade_goods
            .iter()
            // .filter(|trade| trade.r#type != TradeType::Export)
            .min_by_key(|trade| trade.purchase_price)
            .context("No valid market found")?;

        debug!("Selected market: {:?}", best_market);
        Ok(best_market.waypoint_symbol.clone())
    } else {
        market_trades.sort_by(|a, b| a.r#type.cmp(&b.r#type));

        let first_market = market_trades.first().context("No valid market found")?;

        debug!("Selected market: {:?}", first_market);
        Ok(first_market.waypoint_symbol.clone())
    }
}

fn get_updated_procurement(
    contract: &Contract,
    current_procurement: &ContractDeliverGood,
) -> Result<ContractDeliverGood> {
    contract
        .terms
        .deliver
        .as_ref()
        .context("No delivery terms")?
        .iter()
        .find(|p| {
            p.trade_symbol == current_procurement.trade_symbol
                && p.destination_symbol == current_procurement.destination_symbol
        })
        .cloned()
        .context("Procurement not found")
}
