#![recursion_limit = "256"]
mod api;
mod ship;

mod sql;

mod config;
mod tests;

mod control_api;
mod error;
mod manager;
mod pilot;
mod rate_limiter;
mod types;
mod utils;

use std::{collections::HashSet, env, error::Error, sync::Arc, vec};

use chrono::{DateTime, Utc};
use config::CONFIG;
use env_logger::{Env, Target};
use manager::{
    chart_manager::ChartManager,
    construction_manager::ConstructionManager,
    contract_manager::ContractManager,
    mining_manager::MiningManager,
    scrapping_manager::{update_system, ScrappingManager},
    ship_task::ShipTaskHandler,
    trade_manager::TradeManager,
    Manager,
};
use rsntp::AsyncSntpClient;
use ship::ShipManager;
use space_traders_client::models::{self, chart};
use sql::DatabaseConnector;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use types::{ConductorContext, WaypointCan};

use crate::api::Api;
use log::{debug, error, info, warn};

use std::num::NonZeroU32;

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (context, manager_token, managers) = setup_context().await?;

    // run_conductor(context.clone()).await?;

    let erg = start(context, manager_token, managers).await;

    if let Err(errror) = erg {
        println!("Main error: {} {}", errror, errror);
    }

    Ok(())
}

async fn setup_unauthed() -> Result<(Api, sql::DbPool), anyhow::Error> {
    dotenvy::dotenv()?;

    let env = Env::default()
        .filter_or("RUST_LOG", "info")
        .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::Builder::from_env(env)
        .target(Target::Stdout)
        .init();

    check_time().await;

    let access_token = env::var("ACCESS_TOKEN").ok();
    let database_url = env::var("DATABASE_URL").unwrap();

    info!("{:?}", CONFIG.clone());

    let api: Api = Api::new(access_token, 550, NonZeroU32::new(2).unwrap());

    let database_pool = sql::DbPool::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?,
    );

    Ok((api, database_pool))
}

async fn setup_context(
) -> Result<(ConductorContext, CancellationToken, Vec<Box<dyn Manager>>), anyhow::Error> {
    let (api, database_pool) = setup_unauthed().await?;

    let my_agent = api.get_my_agent().await?;
    info!("My agent: {:?}", my_agent);

    let ships = api.get_all_my_ships(20).await?;
    info!("Ships: {:?}", ships.len());

    let system_symbols = ships
        .iter()
        .map(|s| s.nav.system_symbol.clone())
        .collect::<HashSet<_>>();

    debug!("Systems: {}", system_symbols.len());

    for system_symbol in system_symbols {
        let db_system = sql::System::get_by_id(&database_pool, &system_symbol).await?;
        let waypoints = sql::Waypoint::get_by_system(&database_pool, &system_symbol).await?;

        if db_system.is_none() || waypoints.is_empty() {
            // some systems have no waypoints, but we likely won't have ships there
            update_system(&database_pool, &api, &system_symbol, true).await?;
        }
    }

    let (sender, receiver) = broadcast::channel(1024);

    let ship_manager = Arc::new(ship::ShipManager::new(
        ship::my_ship_update::InterShipBroadcaster { sender, receiver },
    )); // ship::ShipManager::new();

    for ship in ships {
        let mut ship_i = ship::MyShip::from_ship(ship.clone(), ship_manager.get_broadcaster());
        ship::MyShip::update_info_db(ship.clone(), &database_pool).await?;
        ship_i.apply_from_db(database_pool.clone()).await?;
        ShipManager::add_ship(&ship_manager, ship_i).await;
    }

    // manager::scrapping_manager::update_all_systems(&database_pool, &api).await?;

    // let all_gates = sql::Waypoint::get_all(&database_pool)
    //     .await?
    //     .into_iter()
    //     .filter(|w| w.is_jump_gate())
    //     .filter(|w| {
    //         !w.traits
    //             .iter()
    //             .any(|t| *t == models::WaypointTraitSymbol::Uncharted)
    //     })
    //     .map(|w| (w.system_symbol, w.symbol))
    //     .collect::<Vec<_>>();

    // info!("All gates: {}", all_gates.len());

    // let jump_gates = manager::scrapping_manager::get_all_jump_gates(&api, all_gates).await?;

    // info!("JumpGates: {:?}", jump_gates);
    // let erg = manager::scrapping_manager::update_jump_gates(&database_pool, jump_gates).await;
    // if erg.is_err() {
    //     warn!("JumpGate scrapping error: {}", erg.unwrap_err());
    // }

    // panic!();

    let construction_manager_data = ConstructionManager::create();
    let contract_manager_data = ContractManager::create();
    let mining_manager_data = MiningManager::create();
    let scrapping_manager_data = ScrappingManager::create();
    let trade_manager_data = TradeManager::create();
    let chart_manager = ChartManager::create();
    let ship_task_handler = ShipTaskHandler::create();

    let context = ConductorContext {
        api: api.clone(),
        database_pool,
        ship_manager,
        ship_tasks: ship_task_handler.1,
        construction_manager: construction_manager_data.1,
        contract_manager: contract_manager_data.1,
        mining_manager: mining_manager_data.1,
        scrapping_manager: scrapping_manager_data.1,
        trade_manager: trade_manager_data.1,
        chart_manager: chart_manager.1,
    };

    debug!("Context created");

    let main_cancel_token = CancellationToken::new();
    let manager_cancel_token = main_cancel_token.child_token();
    let ship_cancel_token = main_cancel_token.child_token();

    let construction_manager = ConstructionManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        construction_manager_data.0,
    );
    let contract_manager = ContractManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        contract_manager_data.0,
    );
    let mining_manager = MiningManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        mining_manager_data.0,
        mining_manager_data.2,
    );
    let scrapping_manager = ScrappingManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        scrapping_manager_data.0,
    );
    let trade_manager = TradeManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        trade_manager_data.0,
    );

    let chart_manager = ChartManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        chart_manager.0,
    );

    let ship_task_handler = ShipTaskHandler::new(
        ship_cancel_token.clone(),
        manager_cancel_token.clone(),
        manager_cancel_token.child_token(),
        context.clone(),
        ship_task_handler.0,
    );

    let control_api = control_api::server::ControlApiServer::new(
        context.clone(),
        context.ship_manager.get_rx(),
        manager_cancel_token.child_token(),
        ship_cancel_token.clone(),
    );

    debug!("Managers created");

    Ok((
        context,
        manager_cancel_token,
        vec![
            Box::new(construction_manager),
            Box::new(contract_manager),
            Box::new(mining_manager),
            Box::new(scrapping_manager),
            Box::new(trade_manager),
            Box::new(chart_manager),
            Box::new(ship_task_handler),
            Box::new(control_api),
        ],
    ))
}

async fn check_time() {
    let client = AsyncSntpClient::new();
    let result = client.synchronize("pool.ntp.org").await.unwrap();
    let local_time: DateTime<Utc> = result.datetime().into_chrono_datetime().unwrap();
    let time_diff = (local_time - Utc::now()).abs();

    info!(
        "The local time is: {} and it should be: {} and the time diff is: {:?}",
        Utc::now(),
        local_time,
        time_diff.to_std().unwrap()
    );

    if time_diff > chrono::Duration::milliseconds(1000) {
        panic!(
            "The time is not correct, off by: {:?}",
            time_diff.to_std().unwrap()
        );
    }
}

async fn start(
    context: ConductorContext,
    manager_token: CancellationToken,
    managers: Vec<Box<dyn Manager>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Start function started");

    // let _ = main_token.cancelled().await;

    debug!("Starting managers and ships");

    let managers_handles = start_managers(managers).await?;
    start_ships(&context).await?;

    debug!("Managers and ships started");

    let manager_future = wait_managers(managers_handles);

    debug!("Waiting for managers and ships to finish");

    let erg = manager_future.await;

    debug!("Managers and ships finished");

    if let Err(errror) = erg {
        error!("Managers error: {} {:?}", errror, errror);
    }

    debug!("Start function finished");

    Ok(())
}

async fn start_managers(
    managers: Vec<Box<dyn Manager>>,
) -> Result<
    Vec<(
        tokio::task::JoinHandle<Result<(), error::Error>>,
        String,
        CancellationToken,
    )>,
    crate::error::Error,
> {
    let mut handles: Vec<(
        tokio::task::JoinHandle<Result<(), error::Error>>,
        String,
        CancellationToken,
    )> = Vec::new();
    for mut manager in managers {
        let name = manager.get_name().to_string();
        let cancel_token = manager.get_cancel_token().clone();
        debug!("Starting manager {}", name);
        let handle = tokio::task::spawn(async move {
            let erg = manager.run().await;

            if let Err(errror) = &erg {
                error!("Managers error: {} {:?}", errror, errror);
            }

            erg
        });
        handles.push((handle, name, cancel_token));
    }
    Ok(handles)
}

async fn start_ships(context: &ConductorContext) -> Result<(), crate::error::Error> {
    let ship_names: Vec<sql::ShipInfo> = sql::ShipInfo::get_all(&context.database_pool).await?;

    let len = ship_names.len();
    debug!("Starting {} ships", len);

    for ship in ship_names {
        context.ship_tasks.start_ship(ship).await;
    }

    debug!("Started pilots for {} ships", len);

    Ok(())
}

async fn wait_managers(
    managers_handles: Vec<(
        tokio::task::JoinHandle<Result<(), error::Error>>,
        String,
        CancellationToken,
    )>,
) -> Result<(), crate::error::Error> {
    let mut manager_futures = futures::stream::FuturesUnordered::new();

    for handle in managers_handles {
        let manager_name = handle.1;
        let manager_handle = handle.0;
        manager_futures.push(async move {
            let erg = manager_handle.await;
            info!("{:?}: {:?}", manager_name, erg);
            if let Err(errror) = erg {
                log::error!("{:?} manager error: {} {:?}", manager_name, errror, errror);
            } else if let Ok(r_erg) = erg {
                if let Err(errror) = r_erg {
                    log::error!(
                        "{:?} manager error: {} {:?}",
                        manager_name,
                        errror,
                        errror.source(),
                    );
                } else if let Ok(_res) = r_erg {
                }
            }
            manager_name
        });
    }

    while let Some(result) = manager_futures.next().await {
        info!("Manager finished: {}", result);
    }
    Ok(())
}
