mod api;
mod ship;

mod sql;
mod workers;

mod config;
mod tests;

mod control_api;
mod error;
mod manager;
mod pilot;
mod types;

use std::{collections::HashMap, env, error::Error, sync::Arc};

use chrono::{DateTime, Utc};
use config::CONFIG;
use dashmap::DashMap;
use env_logger::{Env, Target};
use manager::{
    construction_manager::ConstructionManager, contract_manager::ContractManager,
    mining_manager::MiningManager, scrapping_manager::ScrappingManager,
    trade_manager::TradeManager, Manager,
};
use rsntp::AsyncSntpClient;
use ship::ShipManager;
use space_traders_client::models::waypoint;
use sql::DatabaseConnector;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use workers::types::Conductor;

use crate::api::Api;
use log::{debug, error, info};

use std::num::NonZeroU32;

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (context, main_token, managers) = setup_context().await?;

    // run_conductor(context.clone()).await?;

    let erg = start(context, main_token, managers).await;

    if let Err(errror) = erg {
        println!("Main error: {} {}", errror, errror);
    }

    Ok(())
}

async fn run_conductor(context: workers::types::ConductorContext) -> anyhow::Result<()> {
    let mut conductors: Vec<Box<dyn Conductor>> = vec![
        workers::construction_fleet::ConstructionFleet::new_box(context.clone()),
        workers::contract_fleet::ContractFleet::new_box(context.clone()),
        workers::mining::mining_fleet::MiningFleet::new_box(context.clone()),
        workers::trading::trading_fleet::TradingFleet::new_box(context.clone()),
        workers::market_scrapers::MarketScraper::new_box(context.clone()),
    ];
    conductors.push(control_api::server::ControlApiServer::new_box(
        context.clone(),
        context.ship_manager.get_rx(),
        conductors
            .iter()
            .map(|c| (c.get_name(), c.is_independent(), c.get_cancel_token()))
            .collect(),
    ));

    let conductor_join_handles = conductors
        .into_iter()
        .map(|mut c| {
            (
                c.get_name(),
                c.is_independent(),
                c.get_cancel_token(),
                tokio::task::spawn(async move { c.run().await }),
            )
        })
        .collect::<Vec<_>>();

    for handle in conductor_join_handles {
        let name = handle.0;
        if !handle.1 {
            handle.2.cancel();
        }
        let erg = handle.3.await;
        println!("{}: {:?}", name, erg);
        if let Err(errror) = erg {
            println!("{} error: {} {:?}", name, errror, errror);
        } else if let Ok(r_erg) = erg {
            if let Err(errror) = r_erg {
                println!(
                    "{} error: {} {:?} {:?} {:?}",
                    name,
                    errror,
                    errror.backtrace(),
                    errror.source(),
                    errror.root_cause()
                );
            } else if let Ok(_res) = r_erg {
            }
        }
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

async fn setup_context() -> Result<
    (
        workers::types::ConductorContext,
        CancellationToken,
        Vec<Box<dyn Manager>>,
    ),
    anyhow::Error,
> {
    let (api, database_pool) = setup_unauthed().await?;

    let my_agent = api.get_my_agent().await?;
    info!("My agent: {:?}", my_agent);

    let ships = api.get_all_my_ships(20).await?;
    info!("Ships: {:?}", ships.len());

    let waypoints = api
        .get_all_waypoints(&ships[0].nav.system_symbol, 20)
        .await?;
    info!("Waypoints: {:?}", waypoints.len());

    sql::Waypoint::insert_bulk(
        &database_pool,
        &waypoints
            .iter()
            .map(sql::Waypoint::from)
            .collect::<Vec<_>>(),
    )
    .await?;

    let ship_manager = Arc::new(ship::ShipManager::new()); // ship::ShipManager::new();

    let (sender, receiver) = broadcast::channel(1024);

    let broadcaster = ship::my_ship_update::InterShipBroadcaster { sender, receiver };

    for ship in ships {
        let mut ship_i = ship::MyShip::from_ship(ship.clone(), broadcaster.clone());

        ship_i.apply_from_db(database_pool.clone()).await.unwrap();

        ShipManager::add_ship(&ship_manager, ship_i).await;
    }

    let all_waypoints: Arc<dashmap::DashMap<String, HashMap<String, waypoint::Waypoint>>> =
        Arc::new(DashMap::new());

    {
        let mut a_wps = all_waypoints
            .entry(waypoints[0].system_symbol.clone())
            .or_default();
        for wp in waypoints.iter() {
            a_wps.insert(wp.symbol.clone(), wp.clone());
        }
    }

    let construction_manager_data = ConstructionManager::create();
    let contract_manager_data = ContractManager::create();
    let mining_manager_data = MiningManager::create();
    let scrapping_manager_data = ScrappingManager::create();
    let trade_manager_data = TradeManager::create();

    let context = workers::types::ConductorContext {
        api: api.clone(),
        database_pool,
        ship_manager,
        all_waypoints: all_waypoints.clone(),
        construction_manager: construction_manager_data.1,
        contract_manager: contract_manager_data.1,
        mining_manager: mining_manager_data.1,
        scrapping_manager: scrapping_manager_data.1,
        trade_manager: trade_manager_data.1,
    };

    let cancel_token = CancellationToken::new();

    let construction_manager = ConstructionManager::new(
        cancel_token.child_token(),
        context.clone(),
        construction_manager_data.0,
    );
    let contract_manager = ContractManager::new(
        cancel_token.child_token(),
        context.clone(),
        contract_manager_data.0,
    );
    let mining_manager = MiningManager::new(
        cancel_token.child_token(),
        context.clone(),
        mining_manager_data.0,
    );
    let scrapping_manager = ScrappingManager::new(
        cancel_token.child_token(),
        context.clone(),
        scrapping_manager_data.0,
    );
    let trade_manager = TradeManager::new(
        cancel_token.child_token(),
        context.clone(),
        trade_manager_data.0,
    );

    let control_api = control_api::server::ControlApiServer::new(
        context.clone(),
        context.ship_manager.get_rx(),
        vec![],
    );

    Ok((
        context,
        cancel_token,
        vec![
            Box::new(construction_manager),
            Box::new(contract_manager),
            Box::new(mining_manager),
            Box::new(scrapping_manager),
            Box::new(trade_manager),
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
    context: workers::types::ConductorContext,
    main_token: CancellationToken,
    managers: Vec<Box<dyn Manager>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Start function started");

    // let _ = main_token.cancelled().await;

    let managers_handles = start_managers(managers).await?;
    let ship_handles = start_ships(context.clone()).await?;

    debug!("Managers and ships started");

    let manager_future = wait_managers(managers_handles);
    let ship_future = wait_ships(ship_handles);

    debug!("Waiting for managers and ships to finish");

    let erg: (Result<(), error::Error>, Result<(), error::Error>) =
        tokio::join!(manager_future, ship_future);

    debug!("Managers and ships finished");

    if let Err(errror) = erg.0 {
        error!("Managers error: {} {}", errror, errror);
    }

    if let Err(errror) = erg.1 {
        error!("Ships error: {} {}", errror, errror);
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
        let handle = tokio::task::spawn(async move { manager.run().await });
        handles.push((handle, name, cancel_token));
    }
    Ok(handles)
}

async fn start_ships(
    context: workers::types::ConductorContext,
) -> Result<
    Vec<(
        sql::ShipInfo,
        tokio::task::JoinHandle<Result<(), anyhow::Error>>,
        CancellationToken,
    )>,
    crate::error::Error,
> {
    let ship_names = sql::ShipInfo::get_all(&context.database_pool).await?;
    let ship_handles: Vec<(
        sql::ShipInfo,
        tokio::task::JoinHandle<Result<(), anyhow::Error>>,
        CancellationToken,
    )> = ship_names
        .into_iter()
        .map(|s| {
            let pilot = pilot::Pilot::new(context.clone(), s.symbol.clone());
            let cancel_token = pilot.get_cancel_token();
            debug!("Starting pilot for ship {}", s.symbol);
            let handle: tokio::task::JoinHandle<Result<(), anyhow::Error>> =
                tokio::task::spawn(async move {
                    let _erg = pilot.pilot_ship().await?;
                    Ok(())
                });
            (s, handle, cancel_token)
        })
        .collect::<Vec<_>>();

    debug!("Started pilots for {} ships", ship_handles.len());

    Ok(ship_handles)
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

async fn wait_ships(
    ship_handles: Vec<(
        sql::ShipInfo,
        tokio::task::JoinHandle<Result<(), anyhow::Error>>,
        CancellationToken,
    )>,
) -> Result<(), crate::error::Error> {
    for handle in ship_handles {
        let ship_name = handle.0;
        let ship_handle = handle.1;
        let erg = ship_handle.await;
        info!("{:?}: {:?}", ship_name, erg);
        if let Err(errror) = erg {
            log::error!("{:?} error: {} {:?}", ship_name, errror, errror);
        } else if let Ok(r_erg) = erg {
            if let Err(errror) = r_erg {
                log::error!(
                    "{:?} error: {} {:?} {:?} {:?}",
                    ship_name,
                    errror,
                    errror.backtrace(),
                    errror.source(),
                    errror.root_cause()
                );
            } else if let Ok(_res) = r_erg {
            }
        }
    }
    Ok(())
}
