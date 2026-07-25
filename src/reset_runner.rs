use std::{collections::HashSet, num::NonZeroU32, str::FromStr, sync::Arc};

use database::DatabaseConnectorAsync;
use ship::ShipManager;
use space_traders_client::models;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use utils::{get_system_symbol, WaypointCan};

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
    utils::{ConductorContext, RunInfo},
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

pub async fn run_reset(
    api_key: &str,
    database_pool: database::DbPool,
    global_cancel_token: &CancellationToken,
    socket_address: String,
) -> Result<ResetSummary, anyhow::Error> {
    let api: space_traders_client::Api =
        space_traders_client::Api::new(Some(api_key.to_string()), 500, NonZeroU32::new(2).unwrap());

    let run_cancel_token = global_cancel_token.child_token();

    let (run_info, my_agent, ships) = get_api_main_info(&api).await?;

    let (context, managers) = init_min_context(api, database_pool).await?;

    let context = populate_context(context, &my_agent, run_info).await?;

    populate_database(&context, &my_agent).await?;

    init_exports_to_imports(&context.api, &context.database_pool).await?;

    init_systems_with_ships(&context, &ships).await?;

    ensure_main_system_fleets(&context).await?;

    let managers = init_managers(run_cancel_token, &context, managers, socket_address).await?;

    setup_ships(&context, ships).await?;

    let managers_handles = managers.start();

    start_ships(&context).await?;

    let manager = managers_handles.wait().await?;
    // wait(managers_handles).await?;

    let run_result = analyze_run(&context, &manager).await?;

    Ok(run_result)
}

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

async fn init_systems_with_ships(
    context: &ConductorContext,
    ships: &[models::Ship],
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

        if db_system.is_none() || waypoints.is_empty() {
            init_system(&context.database_pool, &context.api, &system_symbol).await?;
        }
    }

    Ok(())
}

async fn init_system(
    database_pool: &database::DbPool,
    api: &space_traders_client::Api,
    system_symbol: &str,
) -> Result<(), anyhow::Error> {
    tracing::debug!(system = %system_symbol, "Updating system and waypoints");
    // some systems have no waypoints, but we likely won't have ships there
    scrapping_manager::utils::update_system(database_pool, api, system_symbol, true).await?;
    let wps = database::Waypoint::get_by_system(
        database_pool,
        system_symbol,
        database::PaginatedQuery::unpaged(),
    )
    .await?
    .items
    .into_iter()
    .filter(|w| w.is_marketplace())
    .map(|w| (w.system_symbol, w.symbol, w.is_under_construction))
    .collect::<Vec<_>>();

    let markets = scrapping_manager::utils::get_all_markets(api, &wps).await?;
    let markets_len = markets.len();
    scrapping_manager::utils::update_markets(markets, database_pool.clone()).await?;

    for waypoint in wps.iter().filter(|f| f.2) {
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

    tracing::debug!(system = %system_symbol, waypoints = wps.len(), markets = markets_len, "Updated markets");
    Ok(())
}

async fn init_min_context(
    api: space_traders_client::Api,
    database_pool: database::DbPool,
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

async fn init_managers(
    run_cancel_token: CancellationToken,
    context: &ConductorContext,
    manager_receivers: ManagerReceiver,
    socket_address: String,
) -> Result<ManagerManager, anyhow::Error> {
    let manager_cancel_token = run_cancel_token.child_token();
    let ship_cancel_token = run_cancel_token.child_token();

    let construction_manager = ConstructionManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.construction_manager,
    );
    let contract_manager = ContractManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.contract_manager,
    );
    let mining_manager = MiningManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.mining_manager,
        manager_receivers.transfer_manager,
        context.config.read().await.max_miners_per_waypoint,
    );
    let scrapping_manager = ScrappingManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.scrapping_manager,
    );
    let trade_manager = TradeManager::init(
        manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.trade_manager,
    )
    .await?;

    let chart_manager = ChartManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.chart_manager,
    );

    let fleet_manager = FleetManager::new(
        manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.fleet_manager,
    );

    let ship_task_handler = ShipTaskHandler::new(
        ship_cancel_token.clone(),
        manager_cancel_token.clone(),
        manager_cancel_token.child_token(),
        context.clone(),
        manager_receivers.ship_task,
    );

    let control_api = control_api::server::ControlApiServer::new(
        context.clone(),
        context.ship_manager.get_rx(),
        manager_cancel_token.child_token(),
        ship_cancel_token.clone(),
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
