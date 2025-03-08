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

use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use config::CONFIG;
use dashmap::DashMap;
use env_logger::{Env, Target};
use manager::{
    construction_manager::ConstructionManager, contract_manager::ContractManager,
    mining_manager::MiningManager, scrapping_manager::ScrappingManager, ship_task::ShipTaskHandler,
    trade_manager::TradeManager, Manager,
};
use rsntp::AsyncSntpClient;
use ship::ShipManager;
use space_traders_client::models::waypoint;
use sql::DatabaseConnector;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use types::ConductorContext;

use crate::api::Api;
use log::{debug, error, info};

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

    let all_waypoints: Arc<dashmap::DashMap<String, HashMap<String, waypoint::Waypoint>>> =
        Arc::new(DashMap::new());

    for system_symbol in system_symbols {
        let db_system = sql::System::get_by_id(&database_pool, &system_symbol).await?;
        let waypoints = sql::Waypoint::get_by_system(&database_pool, &system_symbol).await?;
        if db_system.is_none() {
            let system_resp = api.get_system(&system_symbol).await?;
            let system = sql::System::from(&*system_resp.data);
            sql::System::insert(&database_pool, &system).await?;
        }

        if waypoints.is_empty() {
            // some systems have no waypoints, but we likely won't have ships there
            let waypoints = api.get_all_waypoints(&system_symbol, 20).await?;
            sql::Waypoint::insert_bulk(
                &database_pool,
                &waypoints
                    .iter()
                    .map(sql::Waypoint::from)
                    .collect::<Vec<_>>(),
            )
            .await?;
        }
        let waypoints = sql::Waypoint::get_by_system(&database_pool, &system_symbol).await?;

        {
            let mut a_wps = all_waypoints.entry(system_symbol).or_default();
            for wp in waypoints.iter() {
                a_wps.insert(
                    wp.symbol.clone(),
                    Into::<waypoint::Waypoint>::into(wp).clone(),
                );
            }
        }
    }

    debug!("Waypoints: {}", all_waypoints.len());

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

    // panic!();

    let construction_manager_data = ConstructionManager::create();
    let contract_manager_data = ContractManager::create();
    let mining_manager_data = MiningManager::create();
    let scrapping_manager_data = ScrappingManager::create();
    let trade_manager_data = TradeManager::create();
    let ship_task_handler = ShipTaskHandler::create();

    let context = ConductorContext {
        api: api.clone(),
        database_pool,
        ship_manager,
        ship_tasks: ship_task_handler.1,
        all_waypoints: all_waypoints.clone(),
        construction_manager: construction_manager_data.1,
        contract_manager: contract_manager_data.1,
        mining_manager: mining_manager_data.1,
        scrapping_manager: scrapping_manager_data.1,
        trade_manager: trade_manager_data.1,
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
            Box::new(ship_task_handler),
            Box::new(control_api),
        ],
    ))
}

async fn check_time() {
    let client = AsyncSntpClient::new();
    let result = client.synchronize("pool.ntp.org").await.unwrap();
    let local_time: DateTime<Utc> =
        DateTime::from(result.datetime().into_chrono_datetime().unwrap());
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
    for handle in managers_handles {
        let manager_name = handle.1;
        let manager_handle = handle.0;
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
    }
    Ok(())
}
