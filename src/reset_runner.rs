use std::{collections::HashSet, env, num::NonZeroU32, str::FromStr, sync::Arc};

use database::DatabaseConnectorAsync;
use futures::StreamExt;
use ship::ShipManager;
use space_traders_client::models;
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;
use utils::{get_system_symbol, WaypointCan};

use crate::{
    control_api, error,
    manager::{
        self,
        chart_manager::ChartManager,
        construction_manager::ConstructionManager,
        contract_manager::ContractManager,
        fleet_manager::FleetManager,
        mining_manager::MiningManager,
        scrapping_manager::{self, ScrappingManager},
        ship_task::ShipTaskHandler,
        trade_manager::TradeManager,
        Manager,
    },
    utils::{ConductorContext, RunInfo},
};

#[derive(Debug, Clone)]
pub struct ResetSummary {
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub agent_symbol: String,
    pub version: String,
}

pub async fn run_reset(
    api_key: &str,
    database_pool: database::DbPool,
    global_cancel_token: &CancellationToken,
) -> Result<ResetSummary, anyhow::Error> {
    let api: space_traders_client::Api =
        space_traders_client::Api::new(Some(api_key.to_string()), 500, NonZeroU32::new(2).unwrap());

    let my_agent = api.get_my_agent().await?;
    tracing::info!(my_agent = ?my_agent, "Fetched agent info");
    database::Agent::upsert(
        &database_pool,
        &database::Agent::from(*my_agent.data.clone()),
    )
    .await?;
    let status = api.get_status().await?;
    tracing::info!(status = ?status, "Fetched status info");

    let run_info = RunInfo {
        agent_symbol: my_agent.data.symbol.clone(),
        headquarters: my_agent.data.headquarters.clone(),
        starting_faction: models::FactionSymbol::from_str(&my_agent.data.starting_faction)?,
        reset_date: status
            .reset_date
            .clone()
            .parse::<chrono::NaiveDate>()?
            .and_hms_opt(13, 0, 0)
            .unwrap()
            .and_local_timezone(chrono::Utc)
            .unwrap(),
        next_reset_date: status.server_resets.next.clone().parse()?,
        version: status.version.clone(),
    };

    let ships = fetch_and_update_ships(&api, &database_pool).await?;

    let exports_to_imports = api.get_exports_to_imports().await?;

    let mappings = database::ExportImportMapping::generate_mapping(*exports_to_imports.data)?;

    tracing::info!(
        mapping_count = mappings.len(),
        "Generated export-import mappings"
    );

    database::ExportImportMapping::insert_bulk(&database_pool, &mappings).await?;

    tracing::info!("Inserted export-import mappings into the database");

    let (context, _cancel_token, managers) =
        create_context(database_pool, api, run_info, &ships, my_agent.data.credits).await?;

    populate_main_system(&context).await?;

    tracing::debug!("Starting managers and ships");

    let managers_handles = start_managers(managers).await?;
    start_ships(&context).await?;

    tracing::debug!("Managers and ships started");

    let manager_future = wait_managers(managers_handles);

    tracing::debug!("Waiting for managers and ships to finish");

    let erg = manager_future.await;

    tracing::debug!("Managers and ships finished");

    if let Err(errror) = erg {
        // if we get a "universe is beeing reset error" gracefully shutdown ships and managers and return
        tracing::error!(errror = ?errror, error = %errror, "Managers error occurred");
    }

    tracing::debug!("Start function finished");

    let r_info = context.run_info.read().await;

    let summary = ResetSummary {
        start_date: r_info.reset_date,
        end_date: chrono::Utc::now(),
        agent_symbol: r_info.agent_symbol.clone(),
        version: r_info.version.clone(),
    };

    Ok(summary)
}

async fn populate_main_system(context: &ConductorContext) -> Result<(), anyhow::Error> {
    let main_system = { get_system_symbol(&context.run_info.read().await.headquarters) };

    if manager::fleet_manager::fleet_population::is_system_populated(
        &context.database_pool,
        &main_system,
    )
    .await?
    {
        tracing::info!("Main system already populated");
    } else {
        tracing::info!("Populating main system fleets");
        manager::fleet_manager::fleet_population::populate_system(context, &main_system).await?;

        tracing::info!("Populated main system fleets");
    }

    Ok(())
}

async fn fetch_and_update_ships(
    api: &space_traders_client::Api,
    database_pool: &database::DbPool,
) -> Result<Vec<models::Ship>, anyhow::Error> {
    let ships = api.get_all_my_ships(20).await?;
    tracing::info!(count = ships.len(), "Fetched ships count");

    let system_symbols = ships
        .iter()
        .map(|s| s.nav.system_symbol.clone())
        .collect::<HashSet<_>>();

    tracing::debug!(count = system_symbols.len(), "Fetched systems count");

    for system_symbol in system_symbols {
        (
            async {
                let db_system = database::System::get_by_id(database_pool, &system_symbol).await?;
                let waypoints = database::Waypoint::get_by_system(
                    database_pool,
                    &system_symbol,
                    database::PaginatedQuery::unpaged()
                ).await?.items;

                if db_system.is_none() || waypoints.is_empty() {
                    tracing::debug!(system = %system_symbol, "Updating system and waypoints");
                    // some systems have no waypoints, but we likely won't have ships there
                    scrapping_manager::utils::update_system(
                        database_pool,
                        api,
                        &system_symbol,
                        true
                    ).await?;
                    let wps = database::Waypoint
                        ::get_by_system(
                            database_pool,
                            &system_symbol,
                            database::PaginatedQuery::unpaged()
                        ).await?
                        .items.into_iter()
                        .filter(|w| w.is_marketplace())
                        .map(|w| (w.system_symbol, w.symbol, w.is_under_construction))
                        .collect::<Vec<_>>();

                    let markets = scrapping_manager::utils::get_all_markets(api, &wps).await?;
                    let markets_len = markets.len();
                    scrapping_manager::utils::update_markets(markets, database_pool.clone()).await?;

                    for waypoint in wps.iter().filter(|f| f.2) {
                        let construction = api.get_construction(&waypoint.0, &waypoint.1).await?;
                        tracing::debug!("Got construction: {:?}", construction);

                        let materials = construction.data.materials
                            .iter()
                            .map(|m| database::ConstructionMaterial::from(m, &waypoint.1))
                            .collect::<Vec<_>>();

                        database::ConstructionMaterial::insert_bulk(
                            database_pool,
                            &materials
                        ).await?;
                    }

                    tracing::debug!(system = %system_symbol, waypoints = wps.len(), markets = markets_len, "Updated markets");
                }
                Ok::<(), crate::error::Error>(())
            }
        ).instrument(tracing::info_span!("update_system", system=%system_symbol)).await?;
    }
    Ok(ships)
}

async fn create_context(
    database_pool: database::DbPool,
    api: space_traders_client::Api,
    run_info: RunInfo,
    ships: &[models::Ship],
    current_funds: i64,
) -> Result<(ConductorContext, CancellationToken, Vec<Box<dyn Manager>>), anyhow::Error> {
    let (sender, receiver) = broadcast::channel(1024);

    let ship_manager = Arc::new(ship::ShipManager::new(
        ship::my_ship_update::InterShipBroadcaster { sender, receiver },
    ));

    for ship in ships {
        let mut ship_i = ship::MyShip::from_ship(ship.clone(), ship_manager.get_broadcaster());
        ship::MyShip::update_info_db(ship.clone(), &database_pool).await?;
        ship_i.apply_from_db(database_pool.clone()).await?;
        ShipManager::add_ship(&ship_manager, ship_i).await;
    }

    let construction_manager_data = ConstructionManager::create();
    let contract_manager_data = ContractManager::create();
    let mining_manager_data = MiningManager::create();
    let scrapping_manager_data = ScrappingManager::create();
    let trade_manager_data = TradeManager::create();
    let chart_manager = ChartManager::create();
    let fleet_manager = FleetManager::create();
    let ship_task_handler = ShipTaskHandler::create();

    let config: crate::utils::Config =
        toml_edit::de::from_str(&std::fs::read_to_string("config.toml").unwrap()).unwrap();

    let max_miners_per_waypoint = config.max_miners_per_waypoint;

    let budget_manager = manager::budget_manager::BudgetManager::init(
        &database_pool,
        current_funds,
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

    tracing::debug!("Context created");

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
    let trade_manager = TradeManager::init(
        manager_cancel_token.child_token(),
        context.clone(),
        trade_manager_data.0,
    )
    .await?;

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

    let socket_address = env::var("SOCKET_ADDRESS")
        .ok()
        .unwrap_or("0.0.0.0:8780".to_string());

    let control_api = control_api::server::ControlApiServer::new(
        context.clone(),
        context.ship_manager.get_rx(),
        manager_cancel_token.child_token(),
        ship_cancel_token.clone(),
        socket_address,
    );

    tracing::debug!("Managers created");

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
        tracing::debug!(manager_name = %name, "Starting manager");
        let handle = tokio::task::spawn(async move {
            let erg = manager.run().await;

            if let Err(errror) = &erg {
                tracing::error!(errror = ?errror, error = %errror, "Managers error occurred");
            }

            erg
        });
        // let handle = tokio::task::Builder::new()
        //     .name(format!("manager-{}", name).as_str())
        //     .spawn(async move {
        //         let erg = manager.run().await;

        //         if let Err(errror) = &erg {
        //             tracing::error!(errror = ?errror, "Managers error");
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
        database::ShipInfo::get_all(&context.database_pool, database::PaginatedQuery::unpaged())
            .await?
            .items;

    let len = ship_names.len();
    tracing::debug!(ship_count = %len, "Starting ships");

    for ship in ship_names {
        context.ship_tasks.start_ship(ship).await;
    }

    tracing::debug!(ship_count = %len, "Started pilots for ships");

    Ok(())
}

async fn wait_managers(managers_handles: Vec<ManagersHandle>) -> Result<(), crate::error::Error> {
    let mut manager_futures = futures::stream::FuturesUnordered::new();

    for handle in managers_handles {
        let manager_name = handle.1;
        let manager_handle = handle.0;
        manager_futures.push(async move {
            let erg = manager_handle.await;
            tracing::info!(manager_name = %manager_name, erg = ?erg, "Manager finished and joined");
            if let Err(ref errror) = erg {
                tracing::error!(manager_name = %manager_name, error = ?errror, "Join error");
            } else if let Ok(r_erg) = erg {
                if let Err(errror) = r_erg {
                    tracing::error!(manager_name = %manager_name, error = ?errror, "Manager error occurred");
                } else if let Ok(_res) = r_erg {
                }
            }
            manager_name
        });
    }

    while let Some(result) = manager_futures.next().await {
        tracing::info!(result = ?result, "Manager finished");
    }
    Ok(())
}
