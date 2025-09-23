#![recursion_limit = "256"]
mod tests;

mod control_api;
mod error;
mod manager;
mod open_telemetry;
mod pilot;
mod utils;

use std::{collections::HashSet, env, error::Error, str::FromStr, sync::Arc, vec};

use chrono::{DateTime, Utc};
use database::DatabaseConnector;
use manager::{
    chart_manager::ChartManager,
    construction_manager::ConstructionManager,
    contract_manager::ContractManager,
    fleet_manager::FleetManager,
    mining_manager::MiningManager,
    scrapping_manager::{self, ScrappingManager},
    ship_task::ShipTaskHandler,
    trade_manager::TradeManager,
    Manager,
};

use opentelemetry::{
    global::{self, ObjectSafeTracer},
    sdk::propagation::TraceContextPropagator,
};
use opentelemetry_otlp::WithExportConfig;
use rsntp::AsyncSntpClient;
use ship::ShipManager;
use space_traders_client::models;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use ::utils::{RunInfo, WaypointCan};
use log::{debug, error, info};
use tracing::instrument;
use tracing_subscriber::layer::SubscriberExt;
use utils::ConductorContext;

use std::num::NonZeroU32;

use sqlx::postgres::PgPoolOptions;

use crate::open_telemetry::init_trace;

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

async fn setup_unauthed() -> Result<(space_traders_client::Api, database::DbPool), anyhow::Error> {
    dotenvy::dotenv()?;

    // console_subscriber::init();

    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = init_trace().unwrap();
    let fmt_tracer = tracing_subscriber::fmt::layer();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with(fmt_tracer)
        .with(telemetry);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // let env = Env::default()
    //     .filter_or("RUST_LOG", "info")
    //     .write_style_or("RUST_LOG_STYLE", "always");

    // env_logger::Builder::from_env(env)
    //     .target(Target::Stdout)
    //     .init();

    check_time().await;

    let access_token = env::var("ACCESS_TOKEN").ok();
    let database_url = env::var("DATABASE_URL").unwrap();

    let api: space_traders_client::Api =
        space_traders_client::Api::new(access_token, 550, NonZeroU32::new(2).unwrap());

    let database_pool = database::DbPool::new(
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
    let status = api.get_status().await?;
    info!("Status: {:?}", status);

    let run_info = RunInfo {
        agent_symbol: my_agent.data.symbol.clone(),
        headquarters: my_agent.data.headquarters.clone(),
        starting_faction: models::FactionSymbol::from_str(&my_agent.data.starting_faction)?,
        reset_date: status.reset_date.clone().parse()?,
        next_reset_date: status.server_resets.next.clone().parse()?,
        version: status.version.clone(),
    };

    // let ship_purchase = api
    //     .purchase_ship(Some(models::PurchaseShipRequest {
    //         ship_type: models::ShipType::Probe,
    //         waypoint_symbol: "X1-NR44-A2".to_string(),
    //     }))
    //     .await;

    // info!("ss {:?}", ship_purchase);

    // panic!();

    let ships = api.get_all_my_ships(20).await?;
    info!("Ships: {:?}", ships.len());

    let system_symbols = ships
        .iter()
        .map(|s| s.nav.system_symbol.clone())
        .collect::<HashSet<_>>();

    debug!("Systems: {}", system_symbols.len());

    for system_symbol in system_symbols {
        let db_system = database::System::get_by_id(&database_pool, &system_symbol).await?;
        let waypoints = database::Waypoint::get_by_system(&database_pool, &system_symbol).await?;

        if db_system.is_none() || waypoints.is_empty() {
            // some systems have no waypoints, but we likely won't have ships there
            scrapping_manager::utils::update_system(&database_pool, &api, &system_symbol, true)
                .await?;
            let wps = database::Waypoint::get_by_system(&database_pool, &system_symbol)
                .await?
                .into_iter()
                .filter(|w| w.is_marketplace())
                .map(|w| (w.system_symbol, w.symbol))
                .collect::<Vec<_>>();

            let markets = scrapping_manager::utils::get_all_markets(&api, &wps).await?;
            let markets_len = markets.len();
            scrapping_manager::utils::update_markets(markets, database_pool.clone()).await?;
            debug!(
                "Updated markets for system {} {} {}",
                system_symbol,
                wps.len(),
                markets_len
            );
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

    // let gates = database::Waypoint::get_all(&database_pool)
    //     .await?
    //     .into_iter()
    //     .filter(|w| w.is_jump_gate())
    //     .filter(|w| !w.is_charted())
    //     .map(|w| {
    //         let chart = w.is_charted();
    //         (w.system_symbol, w.symbol, chart)
    //     })
    //     .collect::<Vec<_>>();

    // info!("JumpGates: {}", gates.len());

    // let erg = crate::manager::scrapping_manager::utils::get_all_jump_gates(&api, gates).await;
    // if erg.is_err() {
    //     warn!("JumpGate scrapping error: {}", erg.unwrap_err());
    // } else {
    //     let jump_gates = erg.unwrap();
    //     let jump_gates_len = jump_gates.len();
    //     scrapping_manager::utils::update_jump_gates(&database_pool, jump_gates).await?;
    //     debug!("Updated jump gates {}", jump_gates_len);
    // }

    // panic!();

    let construction_manager_data = ConstructionManager::create();
    let contract_manager_data = ContractManager::create();
    let mining_manager_data = MiningManager::create();
    let scrapping_manager_data = ScrappingManager::create();
    let trade_manager_data = TradeManager::create();
    let chart_manager = ChartManager::create();
    let fleet_manager = FleetManager::create();
    let ship_task_handler = ShipTaskHandler::create();

    let config: utils::Config =
        toml_edit::de::from_str(&std::fs::read_to_string("config.toml").unwrap()).unwrap();

    let max_miners_per_waypoint = config.max_miners_per_waypoint;

    let budget_manager = manager::budget_manager::BudgetManager::init(
        &database_pool,
        my_agent.data.credits,
        config.iron_reserve,
    )
    .await?;

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
        fleet_manager: fleet_manager.1,
        chart_manager: chart_manager.1,
        budget_manager: Arc::new(budget_manager),
        run_info: Arc::new(RwLock::new(run_info)),
        config: Arc::new(RwLock::new(config)),
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
        max_miners_per_waypoint,
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

    let fleet_manager = FleetManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        fleet_manager.0,
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
            Box::new(fleet_manager),
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

#[instrument(
    level = "info",
    name = "spacetraders::running",
    skip(context, manager_token, managers)
)]
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

type ManagersHandle = (
    tokio::task::JoinHandle<Result<(), error::Error>>,
    String,
    CancellationToken,
);

async fn start_managers(
    managers: Vec<Box<dyn Manager>>,
) -> Result<Vec<ManagersHandle>, crate::error::Error> {
    let mut handles: Vec<ManagersHandle> = Vec::new();
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
        // let handle = tokio::task::Builder::new()
        //     .name(format!("manager-{}", name).as_str())
        //     .spawn(async move {
        //         let erg = manager.run().await;

        //         if let Err(errror) = &erg {
        //             error!("Managers error: {} {:?}", errror, errror);
        //         }

        //         erg
        //     })
        //     .unwrap();
        handles.push((handle, name, cancel_token));
    }
    Ok(handles)
}

async fn start_ships(context: &ConductorContext) -> Result<(), crate::error::Error> {
    let ship_names: Vec<database::ShipInfo> =
        database::ShipInfo::get_all(&context.database_pool).await?;

    let len = ship_names.len();
    debug!("Starting {} ships", len);

    for ship in ship_names {
        context.ship_tasks.start_ship(ship).await;
    }

    debug!("Started pilots for {} ships", len);

    Ok(())
}

async fn wait_managers(managers_handles: Vec<ManagersHandle>) -> Result<(), crate::error::Error> {
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
