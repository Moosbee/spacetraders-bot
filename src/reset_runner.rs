use std::{collections::HashSet, num::NonZeroU32, str::FromStr, sync::Arc};

use database::DatabaseConnectorAsync;
use ship::ShipManager;
use space_traders_client::models::{self};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use utils::{WaypointCan, get_system_symbol};

use crate::{
    control_api,
    manager::{
        self,
        chart_manager::ChartManager,
        construction_manager::ConstructionManager,
        contract_manager::ContractManager,
        fleet_manager::FleetManager,
        manager_manager::ManagerManager,
        mining_manager::MiningManager,
        scrapping_manager::{self, ScrappingManager},
        ship_task::ShipTaskHandler,
        trade_manager::TradeManager,
    },
    utils::{CancellationTokens, ConductorContext, RunInfo},
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResetSummary {
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub agent_symbol: String,
    pub version: String,
    pub current_funds: i64,
    pub iron_reserve: i64,
    pub reserved_amount: i64,
    pub spendable: i64,
}

#[instrument(skip(api_key, database_pool, global_cancel_token))]
pub async fn run_reset(
    api_key: &str,
    database_pool: database::DbPool,
    global_cancel_token: CancellationToken,
    socket_address: String,
) -> Result<ResetSummary, anyhow::Error> {
    tracing::info!("Starting reset run");

    let api: space_traders_client::Api =
        space_traders_client::Api::new(Some(api_key.to_string()), 500, NonZeroU32::new(2).unwrap());

    let run_cancel_token = global_cancel_token.child_token();

    let (run_info, my_agent, ships) = get_api_main_info(&api).await?;
    tracing::info!(
        agent = %run_info.agent_symbol,
        version = %run_info.version,
        ship_count = ships.len(),
        "Fetched API main info"
    );

    tracing::info!("Initializing minimal context");
    let (context, managers) = init_min_context(
        api,
        database_pool,
        run_cancel_token.clone(),
        global_cancel_token.clone(),
    )
    .await?;

    let context = populate_context(context, &my_agent, run_info).await?;

    tracing::info!("Populating database with agent info");
    populate_database(&context, &my_agent).await?;

    tracing::info!("Initializing export-import mappings");
    init_exports_to_imports(&context.api, &context.database_pool).await?;

    tracing::info!("Initializing systems for ship locations");
    init_systems_with_ships(&context, &ships, false).await?;

    tracing::info!("Ensuring main system fleets are populated");
    ensure_main_system_fleets(&context).await?;

    tracing::info!("Initializing managers");
    let managers = init_managers(&context, managers, socket_address).await?;

    tracing::info!(ship_count = ships.len(), "Setting up ships");
    setup_ships(&context, ships).await?;

    tracing::info!("Starting managers");
    let managers_handles = managers.start();

    tracing::info!("Starting ship pilots");
    start_ships(&context).await?;

    tracing::info!("Waiting for managers to complete");
    let manager = managers_handles
        .wait(&global_cancel_token, &run_cancel_token)
        .await?;

    tracing::info!("Analyzing run results");
    let run_result = analyze_run(&context, &manager).await?;

    tracing::info!(
        agent = %run_result.agent_symbol,
        funds = run_result.current_funds,
        spendable = run_result.spendable,
        "Reset run completed successfully"
    );

    Ok(run_result)
}

#[instrument(skip(context, _manager))]
async fn analyze_run(
    context: &ConductorContext,
    _manager: &ManagerManager,
) -> Result<ResetSummary, anyhow::Error> {
    let run_info = context.run_info.read().await;
    let money = context.budget_manager.get_budget_info().await;

    Ok(ResetSummary {
        start_date: run_info.reset_date,
        end_date: chrono::Utc::now(),
        agent_symbol: run_info.agent_symbol.clone(),
        version: run_info.version.clone(),
        current_funds: money.current_funds,
        iron_reserve: money.iron_reserve,
        reserved_amount: money.reserved_amount,
        spendable: money.spendable,
    })
}

#[instrument(skip(context, ships))]
async fn setup_ships(
    context: &ConductorContext,
    ships: Vec<models::Ship>,
) -> Result<(), anyhow::Error> {
    let ship_manager = context.ship_manager.clone();
    for ship in ships {
        let mut ship_i = ship::MyShip::from_ship(ship.clone(), ship_manager.get_broadcaster());
        ship::MyShip::update_info_db(ship.clone(), &context.database_pool).await?;
        ship_i.apply_from_db(context.database_pool.clone()).await?;
        ShipManager::add_ship(&ship_manager, ship_i).await;
    }

    Ok(())
}

#[instrument(skip(api))]
async fn get_api_main_info(
    api: &space_traders_client::Api,
) -> Result<(RunInfo, models::Agent, Vec<models::Ship>), anyhow::Error> {
    let my_agent = api.get_my_agent().await?;
    let status = api.get_status().await?;

    let ships = api.get_all_my_ships(20).await?;

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

    Ok((run_info, *my_agent.data, ships))
}

#[instrument(skip(context, my_agent))]
async fn populate_context(
    mut context: ConductorContext,
    my_agent: &models::Agent,
    run_info: RunInfo,
) -> Result<ConductorContext, anyhow::Error> {
    let config: crate::utils::Config =
        toml_edit::de::from_str(&std::fs::read_to_string("config.toml").unwrap()).unwrap();

    let mut write_config = context.config.write().await;
    let iron_reserve = config.iron_reserve;
    *write_config = config.clone();
    drop(write_config);

    let mut write_run_info = context.run_info.write().await;
    *write_run_info = run_info.clone();
    drop(write_run_info);

    let mut budget_manager = (*context.budget_manager).duplicate().await;
    budget_manager
        .load(&context.database_pool, my_agent.credits, iron_reserve)
        .await?;

    context.budget_manager = Arc::new(budget_manager);

    Ok(context)
}

#[instrument(skip(api, database_pool))]
async fn init_exports_to_imports(
    api: &space_traders_client::Api,
    database_pool: &database::DbPool,
) -> Result<(), anyhow::Error> {
    let exports_to_imports: models::GetSupplyChain200Response =
        api.get_exports_to_imports().await?;
    let mappings = database::ExportImportMapping::generate_mapping(*exports_to_imports.data)?;

    tracing::info!(
        mapping_count = mappings.len(),
        "Generated export-import mappings"
    );

    database::ExportImportMapping::insert_bulk(database_pool, &mappings).await?;
    tracing::info!("Inserted export-import mappings into the database");
    Ok(())
}
#[instrument(skip(context, my_agent))]
async fn populate_database(
    context: &ConductorContext,
    my_agent: &models::Agent,
) -> Result<(), anyhow::Error> {
    database::Agent::upsert(
        &context.database_pool,
        &database::Agent::from(my_agent.clone()),
    )
    .await?;

    Ok(())
}

#[instrument(skip(context))]
async fn ensure_main_system_fleets(context: &ConductorContext) -> Result<(), anyhow::Error> {
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

#[instrument(skip(context, ships))]
async fn init_systems_with_ships(
    context: &ConductorContext,
    ships: &[models::Ship],
    force: bool,
) -> Result<(), anyhow::Error> {
    let system_symbols = ships
        .iter()
        .map(|s| s.nav.system_symbol.clone())
        .collect::<HashSet<_>>();

    tracing::debug!(count = system_symbols.len(), "Fetched systems count");

    for system_symbol in system_symbols {
        let db_system = database::System::get_by_id(&context.database_pool, &system_symbol).await?;
        let waypoints = database::Waypoint::get_by_system(
            &context.database_pool,
            &system_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        if db_system.is_none() || waypoints.is_empty() || force {
            init_system(&context.database_pool, &context.api, &system_symbol).await?;
        }
    }

    Ok(())
}

#[instrument(skip(database_pool, api))]
async fn init_system(
    database_pool: &database::DbPool,
    api: &space_traders_client::Api,
    system_symbol: &str,
) -> Result<(), anyhow::Error> {
    tracing::debug!(system = %system_symbol, "Updating system and waypoints");
    // some systems have no waypoints, but we likely won't have ships there
    scrapping_manager::utils::update_system(database_pool, api, system_symbol, true).await?;

    let waypoints = database::Waypoint::get_by_system(
        database_pool,
        system_symbol,
        database::PaginatedQuery::unpaged(),
    )
    .await?
    .items;
    let marketplaces = waypoints
        .iter()
        .filter(|w| w.is_marketplace())
        .map(|w| {
            (
                w.system_symbol.clone(),
                w.symbol.clone(),
                w.is_under_construction,
            )
        })
        .collect::<Vec<_>>();

    let markets = scrapping_manager::utils::get_all_markets(api, &marketplaces).await?;
    let markets_len = markets.len();
    scrapping_manager::utils::update_markets(markets, database_pool.clone()).await?;

    for waypoint in marketplaces.iter().filter(|f| f.2) {
        let construction = api.get_construction(&waypoint.0, &waypoint.1).await?;
        tracing::debug!("Got construction: {:?}", construction);

        let materials = construction
            .data
            .materials
            .iter()
            .map(|m| database::ConstructionMaterial::from(m, &waypoint.1))
            .collect::<Vec<_>>();

        database::ConstructionMaterial::insert_bulk(database_pool, &materials).await?;
    }

    let jump_gates = waypoints
        .iter()
        .filter(|w| w.is_jump_gate())
        .map(|w| (w.system_symbol.clone(), w.symbol.clone(), w.is_charted()))
        .collect::<Vec<_>>();

    let jump_gates = scrapping_manager::utils::get_all_jump_gates(api, jump_gates).await?;
    let jump_gates_len = jump_gates.len();
    scrapping_manager::utils::update_jump_gates(database_pool, jump_gates).await?;

    tracing::debug!(system = %system_symbol, waypoints = marketplaces.len(), markets = markets_len, jump_gates = jump_gates_len, "Updated markets, jump gates and waypoints for system");
    Ok(())
}

#[instrument(skip(api, database_pool, run_cancel_token, global_cancel_token))]
async fn init_min_context(
    api: space_traders_client::Api,
    database_pool: database::DbPool,
    run_cancel_token: CancellationToken,
    global_cancel_token: CancellationToken,
) -> Result<(ConductorContext, ManagerReceiver), anyhow::Error> {
    let ship_manager = Arc::new(ship::ShipManager::new(
        ship::my_ship_update::InterShipBroadcaster::new(1024),
    ));

    let construction_manager_data = ConstructionManager::create();
    let contract_manager_data = ContractManager::create();
    let mining_manager_data = MiningManager::create();
    let scrapping_manager_data = ScrappingManager::create();
    let trade_manager_data = TradeManager::create();
    let chart_manager = ChartManager::create();
    let fleet_manager = FleetManager::create();
    let ship_task_handler = ShipTaskHandler::create();

    let budget_manager = manager::budget_manager::BudgetManager::default();

    // let fast_manager_cancel_token= run_cancel_token.child_token();
    // let fast_ship_cancel_token = run_cancel_token.child_token();

    let cancellation_tokens = CancellationTokens {
        global_cancel_token,
        fast_manager_cancel_token: run_cancel_token.child_token(),
        slow_manager_cancel_token: run_cancel_token.child_token(),
        fast_ship_cancel_token: run_cancel_token.child_token(),
        slow_ship_cancel_token: run_cancel_token.child_token(),
        run_cancel_token,
    };

    let context = ConductorContext {
        api,
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
        run_info: Arc::new(RwLock::new(RunInfo::default())),
        config: Arc::new(RwLock::new(crate::utils::Config::default())),
        cancellation_tokens: Arc::new(cancellation_tokens),
    };

    let manager_receiver = ManagerReceiver {
        construction_manager: construction_manager_data.0,
        contract_manager: contract_manager_data.0,
        mining_manager: mining_manager_data.0,
        scrapping_manager: scrapping_manager_data.0,
        trade_manager: trade_manager_data.0,
        chart_manager: chart_manager.0,
        fleet_manager: fleet_manager.0,
        ship_task: ship_task_handler.0,
        transfer_manager: mining_manager_data.2,
    };

    Ok((context, manager_receiver))
}

struct ManagerReceiver {
    construction_manager: manager::construction_manager::ConstructionManagerReceiver,
    contract_manager: manager::contract_manager::ContractManagerReceiver,
    mining_manager: manager::mining_manager::MiningManagerReceiver,
    scrapping_manager: manager::scrapping_manager::ScrappingManagerReceiver,
    trade_manager: manager::trade_manager::TradeManagerReceiver,
    chart_manager: manager::chart_manager::ChartManagerReceiver,
    fleet_manager: manager::fleet_manager::FleetManagerReceiver,
    ship_task: manager::ship_task::ShipTaskHandlerReceiver,
    transfer_manager: Arc<manager::mining_manager::TransferManager>,
}

#[instrument(skip(context, manager_receivers, socket_address))]
async fn init_managers(
    context: &ConductorContext,
    manager_receivers: ManagerReceiver,
    socket_address: String,
) -> Result<ManagerManager, anyhow::Error> {
    let slow_manager_cancel_token = &context.cancellation_tokens.slow_manager_cancel_token;
    let fast_manager_cancel_token = &context.cancellation_tokens.fast_manager_cancel_token;
    let slow_ship_cancel_token = &context.cancellation_tokens.slow_ship_cancel_token;
    let fast_ship_cancel_token = &context.cancellation_tokens.fast_ship_cancel_token;

    let construction_manager = ConstructionManager::new(
        fast_manager_cancel_token.child_token(),
        slow_manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.construction_manager,
    );
    let contract_manager = ContractManager::new(
        fast_manager_cancel_token.child_token(),
        slow_manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.contract_manager,
    );
    let mining_manager = MiningManager::new(
        fast_manager_cancel_token.child_token(),
        slow_manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.mining_manager,
        manager_receivers.transfer_manager,
        context.config.read().await.max_miners_per_waypoint,
    );
    let scrapping_manager = ScrappingManager::new(
        fast_manager_cancel_token.child_token(),
        slow_manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.scrapping_manager,
    );
    let trade_manager = TradeManager::init(
        fast_manager_cancel_token.child_token(),
        slow_manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.trade_manager,
    )
    .await?;

    let chart_manager = ChartManager::new(
        fast_manager_cancel_token.child_token(),
        slow_manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.chart_manager,
    );

    let fleet_manager = FleetManager::new(
        fast_manager_cancel_token.child_token(),
        slow_manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.fleet_manager,
    );

    let ship_task_handler = ShipTaskHandler::new(
        fast_ship_cancel_token.clone(),
        slow_ship_cancel_token.clone(),
        fast_manager_cancel_token.child_token(),
        fast_manager_cancel_token.child_token(),
        slow_manager_cancel_token.clone(),
        context.clone(),
        manager_receivers.ship_task,
    );

    let control_api = control_api::server::ControlApiServer::new(
        context.clone(),
        context.ship_manager.get_rx(),
        fast_manager_cancel_token.child_token(),
        socket_address,
    );

    let manager_manager = ManagerManager::new(
        construction_manager,
        contract_manager,
        mining_manager,
        scrapping_manager,
        trade_manager,
        fleet_manager,
        chart_manager,
        ship_task_handler,
        control_api,
    );

    Ok(manager_manager)
}

#[instrument(skip(context))]
async fn start_ships(context: &ConductorContext) -> Result<(), anyhow::Error> {
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
