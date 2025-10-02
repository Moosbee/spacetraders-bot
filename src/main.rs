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
    global::{self},
    sdk::propagation::TraceContextPropagator,
};
use rsntp::AsyncSntpClient;
use ship::ShipManager;
use space_traders_client::models;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

use ::utils::{RunInfo, WaypointCan};
use tracing::debug;
use tracing::{instrument, Instrument};
use tracing_subscriber::layer::SubscriberExt;
use utils::ConductorContext;

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

async fn setup_unauthed() -> Result<(space_traders_client::Api, database::DbPool), anyhow::Error> {
    dotenvy::dotenv()?;

    // console_subscriber::init();

    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = open_telemetry::init_trace().unwrap();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let fmt_tracer = tracing_subscriber::fmt::layer();

    // let tracing_tracy = tracing_tracy::TracyLayer::default();

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with(fmt_tracer)
        // .with(tracing_tracy)
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

    // database::ShipInfo::insert(
    //     &database_pool,
    //     &database::ShipInfo {
    //         symbol: "MOSE".to_string(),
    //         display_name: "tester".to_string(),
    //         role: database::ShipInfoRole::Manuel,
    //         active: true,
    //     },
    // )
    // .await?;

    Ok((api, database_pool))
}

async fn setup_context(
) -> Result<(ConductorContext, CancellationToken, Vec<Box<dyn Manager>>), anyhow::Error> {
    let (api, database_pool) = setup_unauthed().await?;

    let my_agent = api.get_my_agent().await?;
    tracing::info!(
      my_agent=?my_agent,
      "My agent"
    );
    let status = api.get_status().await?;
    tracing::info!(
      status=?status,
      "Status"
    );

    let run_info = RunInfo {
        agent_symbol: my_agent.data.symbol.clone(),
        headquarters: my_agent.data.headquarters.clone(),
        starting_faction: models::FactionSymbol::from_str(&my_agent.data.starting_faction)?,
        reset_date: status.reset_date.clone().parse()?,
        next_reset_date: status.server_resets.next.clone().parse()?,
        version: status.version.clone(),
    };

    let ships = async {
        let ships = api.get_all_my_ships(20).await?;
        tracing::info!(count = ships.len(), "Ships count");

        let system_symbols = ships
            .iter()
            .map(|s| s.nav.system_symbol.clone())
            .collect::<HashSet<_>>();

        tracing::debug!(count = system_symbols.len(), "Systems count");

        for system_symbol in system_symbols {
            async {
                let db_system = database::System::get_by_id(&database_pool, &system_symbol).await?;
                let waypoints =
                    database::Waypoint::get_by_system(&database_pool, &system_symbol).await?;

                if db_system.is_none() || waypoints.is_empty() {
                    tracing::debug!(system=%system_symbol, "Updating system and waypoints");
                    // some systems have no waypoints, but we likely won't have ships there
                    scrapping_manager::utils::update_system(
                        &database_pool,
                        &api,
                        &system_symbol,
                        true,
                    )
                    .await?;
                    let wps = database::Waypoint::get_by_system(&database_pool, &system_symbol)
                        .await?
                        .into_iter()
                        .filter(|w| w.is_marketplace())
                        .map(|w| (w.system_symbol, w.symbol))
                        .collect::<Vec<_>>();

                    let markets = scrapping_manager::utils::get_all_markets(&api, &wps).await?;
                    let markets_len = markets.len();
                    scrapping_manager::utils::update_markets(markets, database_pool.clone())
                        .await?;
                    tracing::debug!(
                        system=%system_symbol,
                        waypoints=?wps.len(),
                        markets=?markets_len,
                        "Updated markets"
                    );
                }
                Ok::<(), crate::error::Error>(())
            }
            .instrument(tracing::info_span!("update_system", system=%system_symbol))
            .await?;
        }
        crate::error::Result::Ok(ships)
    }
    .instrument(tracing::info_span!("setup_ship_systems"))
    .await?;

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

    tracing::info!(
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
        tracing::error!("Managers error: {} {:?}", errror, errror);
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
                tracing::error!("Managers error: {} {:?}", errror, errror);
            }

            erg
        });
        // let handle = tokio::task::Builder::new()
        //     .name(format!("manager-{}", name).as_str())
        //     .spawn(async move {
        //         let erg = manager.run().await;

        //         if let Err(errror) = &erg {
        //             tracing::error!("Managers error: {} {:?}", errror, errror);
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
            tracing::info!(
                manager_name,
                erg=?erg,
                "Manager finished and joined"
            );
            if let Err(ref errror) = erg {
                tracing::error!(manager_name, error = format!("{:?}", errror), "Join error");
            } else if let Ok(r_erg) = erg {
                if let Err(errror) = r_erg {
                    tracing::error!(
                        manager_name,
                        error = format!("{:?}", errror),
                        error = format!("{:?}", errror.source()),
                        "Manager error",
                    );
                } else if let Ok(_res) = r_erg {
                }
            }
            manager_name
        });
    }

    while let Some(result) = manager_futures.next().await {
        tracing::info!(
          result=?result,
          "Manager finished"
        );
    }
    Ok(())
}
