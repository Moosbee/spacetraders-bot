use crate::{
    control_api::{graphql::gql_models::GQLShipInfo, GraphiQLError},
    utils::ConductorContext,
};
use async_graphql::{Context, Object};
use database::DatabaseConnector;
use space_traders_client::models;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Edit the in-memory config (and return the updated config). Fields that are None are left unchanged.
    async fn edit_config<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: InputConfig,
    ) -> super::Result<crate::utils::Config> {
        let context = ctx.data::<ConductorContext>()?;
        {
            let mut w = context.config.write().await;
            let mut cfg = w.clone();

            if let Some(v) = input.socket_address {
                cfg.socket_address = v;
            }
            if let Some(v) = input.control_start_sleep {
                cfg.control_start_sleep = v;
            }
            if let Some(v) = input.control_active {
                cfg.control_active = v;
            }

            if let Some(v) = input.scrapper_start_sleep {
                cfg.scrapper_start_sleep = v;
            }
            if let Some(v) = input.scrap_agents {
                cfg.scrap_agents = v;
            }
            if let Some(v) = input.update_all_systems {
                cfg.update_all_systems = v;
            }

            if let Some(v) = input.max_miners_per_waypoint {
                cfg.max_miners_per_waypoint = v;
            }
            if let Some(v) = input.mining_eject_list {
                cfg.mining_eject_list = v;
            }
            if let Some(v) = input.mining_prefer_list {
                cfg.mining_prefer_list = v;
            }
            if let Some(v) = input.ignore_engineered_asteroids {
                cfg.ignore_engineered_asteroids = v;
            }
            if let Some(v) = input.unstable_since_timeout {
                cfg.unstable_since_timeout = v;
            }
            if let Some(v) = input.stop_all_unstable {
                cfg.stop_all_unstable = v;
            }
            if let Some(v) = input.extra_mining_transporter {
                cfg.extra_mining_transporter = v;
            }

            if let Some(v) = input.fuel_cost {
                cfg.fuel_cost = v;
            }
            if let Some(v) = input.antimatter_price {
                cfg.antimatter_price = v;
            }
            if let Some(v) = input.purchase_multiplier {
                cfg.purchase_multiplier = v;
            }

            if let Some(v) = input.market_blacklist {
                cfg.market_blacklist = v;
            }

            if let Some(v) = input.default_purchase_price {
                cfg.default_purchase_price = v;
            }
            if let Some(v) = input.default_sell_price {
                cfg.default_sell_price = v;
            }
            if let Some(v) = input.default_profit {
                cfg.default_profit = v;
            }

            if let Some(v) = input.markup_percentage {
                cfg.markup_percentage = v;
            }
            if let Some(v) = input.margin_percentage {
                cfg.margin_percentage = v;
            }

            if let Some(v) = input.markets_per_ship {
                cfg.markets_per_ship = v;
            }

            if let Some(v) = input.mining_waypoints_per_system {
                cfg.mining_waypoints_per_system = v;
            }
            if let Some(v) = input.mining_ships_per_waypoint {
                cfg.mining_ships_per_waypoint = v;
            }
            if let Some(v) = input.transport_capacity_per_waypoint {
                cfg.transport_capacity_per_waypoint = v;
            }

            if let Some(v) = input.trade_mode {
                cfg.trade_mode = v;
            }
            if let Some(v) = input.trade_profit_threshold {
                cfg.trade_profit_threshold = v;
            }

            if let Some(v) = input.ship_purchase_percentile {
                cfg.ship_purchase_percentile = v;
            }
            if let Some(v) = input.ship_purchase_stop {
                cfg.ship_purchase_stop = v;
            }
            if let Some(v) = input.expand {
                cfg.expand = v;
            }
            if let Some(v) = input.ship_purchase_amount {
                cfg.ship_purchase_amount = v;
            }

            if let Some(v) = input.iron_reserve {
                cfg.iron_reserve = v;
            }

            *w = cfg.clone();
        }

        let old_string = tokio::fs::read_to_string("config.toml")
            .await
            .map_err(|e| GraphiQLError::IO(e.to_string()))?;

        let info = { context.config.read().await.clone() };

        let mut toml_edit_doc = old_string
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| GraphiQLError::IO(e.to_string()))?;

        let config_doc: toml_edit::DocumentMut =
            toml_edit::ser::to_document(&info).map_err(|e| GraphiQLError::IO(e.to_string()))?;

        toml_edit_doc.extend(config_doc.iter());

        let toml_string = toml_edit_doc.to_string();
        tokio::fs::write("config.toml", toml_string)
            .await
            .map_err(|e| GraphiQLError::IO(e.to_string()))?;

        Ok(info)
    }

    /// Add a new fleet for a given system. Returns the created DB fleet.
    async fn add_fleet<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        system_symbol: String,
        active: bool,
        config: database::FleetConfig,
    ) -> super::Result<database::Fleet> {
        let context = ctx.data::<ConductorContext>()?;

        let mut new_fleet = database::Fleet::new(system_symbol, active).with_config(config);
        let inserted_fleet_id =
            database::Fleet::insert_new(&context.database_pool, &new_fleet).await?;
        new_fleet.id = inserted_fleet_id;
        Ok(new_fleet)
    }

    /// Remove a fleet.
    async fn remove_fleet<'ctx>(&self, ctx: &Context<'ctx>, id: i32) -> super::Result<bool> {
        let context = ctx.data::<ConductorContext>()?;
        database::Fleet::delete_by_id(&context.database_pool, id).await?;
        Ok(true)
    }

    /// Edit basic fleet attributes and configuration. Fields that are None are left unchanged.
    async fn edit_fleet<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        id: i32,
        system_symbol: Option<String>,
        active: Option<bool>,
        config: Option<InputFleetConfig>,
    ) -> super::Result<database::Fleet> {
        let context = ctx.data::<ConductorContext>()?;

        // Update non-config fields via database helpers so SQL stays in the database crate
        database::Fleet::update_basic(&context.database_pool, id, system_symbol, active)
            .await
            .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;

        // Update configuration fields selectively per config variant
        if let Some(cfg) = config {
            match cfg {
                InputFleetConfig::Trading(cfg) => {
                    database::Fleet::update_trading_config(
                        &context.database_pool,
                        id,
                        cfg.market_blacklist,
                        cfg.market_prefer_list,
                        cfg.purchase_multiplier,
                        cfg.ship_market_ratio,
                        cfg.min_cargo_space,
                        cfg.trade_mode,
                        cfg.trade_profit_threshold,
                    )
                    .await
                    .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
                }
                InputFleetConfig::Scraping(cfg) => {
                    database::Fleet::update_scraping_config(
                        &context.database_pool,
                        id,
                        cfg.ship_market_ratio,
                        cfg.allowed_requests,
                        cfg.notify_on_shipyard,
                    )
                    .await
                    .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
                }
                InputFleetConfig::Mining(cfg) => {
                    database::Fleet::update_mining_config(
                        &context.database_pool,
                        id,
                        cfg.mining_eject_list,
                        cfg.mining_prefer_list,
                        cfg.ignore_engineered_asteroids,
                        cfg.stop_all_unstable,
                        cfg.unstable_since_timeout,
                        cfg.mining_waypoints,
                        cfg.syphon_waypoints,
                        cfg.miners_per_waypoint,
                        cfg.siphoners_per_waypoint,
                        cfg.surveyers_per_waypoint,
                        cfg.mining_transporters_per_waypoint,
                        cfg.min_transporter_cargo_space,
                        cfg.min_mining_cargo_space,
                        cfg.min_siphon_cargo_space,
                    )
                    .await
                    .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
                }
                InputFleetConfig::Charting(cfg) => {
                    database::Fleet::update_charting_config(
                        &context.database_pool,
                        id,
                        cfg.charting_probe_count,
                    )
                    .await
                    .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
                }
                InputFleetConfig::Construction(cfg) => {
                    database::Fleet::update_construction_config(
                        &context.database_pool,
                        id,
                        cfg.construction_ship_count,
                        cfg.construction_waypoint,
                    )
                    .await
                    .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
                }
                InputFleetConfig::Contract(cfg) => {
                    database::Fleet::update_contract_config(
                        &context.database_pool,
                        id,
                        cfg.contract_ship_count,
                    )
                    .await
                    .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
                }
                InputFleetConfig::Manuel(_cfg) => {
                    // set fleet type to Manuel
                    sqlx::query!(
                        r#"UPDATE fleet SET fleet_type = $1::fleet_type, updated_at = NOW() WHERE id = $2"#,
                        database::FleetType::Manuel as database::FleetType,
                        id
                    )
                    .execute(&context.database_pool.database_pool)
                    .await
                    .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
                }
            }
        }

        // Re-fetch and return the updated fleet
        let updated = database::Fleet::get_by_id(&context.database_pool, id)
            .await?
            .ok_or(super::GraphiQLError::NotFound)?;

        Ok(updated)
    }

    /// Trigger a regeneration of fleet assignments. This will ask the FleetManager to rebuild assignments.
    async fn regenerate_fleet_assignments<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        by: Option<RegenFleetBy>,
    ) -> super::Result<bool> {
        let context = ctx.data::<ConductorContext>()?;
        // fleet_manager has a messenger - ask it to regenerate
        if let Some(by) = by {
            match by {
                RegenFleetBy::System(system_symbol) => {
                    // ask the fleet manager to regenerate assignments for a single system
                    context
                        .fleet_manager
                        .regenerate_system_assignments(system_symbol)
                        .await
                        .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
                }
                RegenFleetBy::Fleet(fleet_id) => {
                    // ask the fleet manager to regenerate assignments for a single fleet
                    context
                        .fleet_manager
                        .regenerate_fleet_assignments(fleet_id)
                        .await
                        .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
                }
            }
        } else {
            // ask the fleet manager to regenerate all assignments
            context
                .fleet_manager
                .regenerate_all_assignments()
                .await
                .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
        }
        Ok(true)
    }

    /// Repopulate a system with some default fleets (convenience helper). Creates a small manual fleet for the system.
    async fn repopulate_system_with_fleets<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        system: String,
    ) -> super::Result<bool> {
        let context = ctx.data::<ConductorContext>()?;
        context
            .fleet_manager
            .populate_system(system)
            .await
            .map_err(|e| super::GraphiQLError::IO(e.to_string()))?;
        Ok(true)
    }

    /// Blacklist a system from population
    async fn blacklist_system<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        system: String,
    ) -> super::Result<bool> {
        let context = ctx.data::<ConductorContext>()?;
        database::System::set_population_disabled_led(&context.database_pool, &system, true)
            .await?;
        Ok(true)
    }

    /// Remove a system from the blacklist
    async fn deblacklist_system<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        system: String,
    ) -> super::Result<bool> {
        let context = ctx.data::<ConductorContext>()?;
        database::System::set_population_disabled_led(&context.database_pool, &system, false)
            .await?;
        Ok(true)
    }

    /// Force assign a ship a new assignment from the fleet manager.
    async fn force_assign_ship<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        ship_symbol: String,
        assignment_id: i64,
        temp: bool,
    ) -> super::Result<GQLShipInfo> {
        let context = ctx.data::<ConductorContext>()?;
        let mut ship = database::ShipInfo::get_by_symbol(&context.database_pool, &ship_symbol)
            .await?
            .ok_or(super::GraphiQLError::NotFound)?;
        if temp {
            ship.temp_assignment_id = Some(assignment_id);
        } else {
            ship.assignment_id = Some(assignment_id);
        }
        database::ShipInfo::insert(&context.database_pool, &ship).await?;
        Ok(ship.into())
    }

    /// Pause a ship (set active=false in ship_info)
    async fn pause_ship<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        ship_symbol: String,
    ) -> super::Result<bool> {
        let context = ctx.data::<ConductorContext>()?;
        if let Some(mut info) =
            database::ShipInfo::get_by_symbol(&context.database_pool, &ship_symbol).await?
        {
            info.active = false;
            database::ShipInfo::insert(&context.database_pool, &info).await?;
            Ok(true)
        } else {
            Err(super::GraphiQLError::NotFound)
        }
    }

    /// Resume a ship (set active=true in ship_info)
    async fn resume_ship<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        ship_symbol: String,
    ) -> super::Result<bool> {
        let context = ctx.data::<ConductorContext>()?;
        if let Some(mut info) =
            database::ShipInfo::get_by_symbol(&context.database_pool, &ship_symbol).await?
        {
            info.active = true;
            database::ShipInfo::insert(&context.database_pool, &info).await?;
            Ok(true)
        } else {
            Err(super::GraphiQLError::NotFound)
        }
    }
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum RegenFleetBy {
    System(String),
    Fleet(i32),
}

#[derive(Debug, Clone, async_graphql::InputObject)]
struct InputConfig {
    pub socket_address: Option<String>,
    pub control_start_sleep: Option<u64>,
    pub control_active: Option<bool>,

    pub scrapper_start_sleep: Option<u64>,
    pub scrap_agents: Option<bool>,
    pub update_all_systems: Option<bool>,

    pub max_miners_per_waypoint: Option<u32>,
    pub mining_eject_list: Option<Vec<models::TradeSymbol>>,
    pub mining_prefer_list: Option<Vec<models::TradeSymbol>>,
    pub ignore_engineered_asteroids: Option<bool>,
    pub unstable_since_timeout: Option<i64>, // in seconds
    pub stop_all_unstable: Option<bool>,
    pub extra_mining_transporter: Option<i32>,

    pub fuel_cost: Option<i32>,
    pub antimatter_price: Option<i32>,
    pub purchase_multiplier: Option<f32>,

    pub market_blacklist: Option<Vec<models::TradeSymbol>>,

    pub default_purchase_price: Option<i32>,
    pub default_sell_price: Option<i32>,
    pub default_profit: Option<i32>,

    // Markup and margin percentages (as decimals)
    pub markup_percentage: Option<f32>,
    pub margin_percentage: Option<f32>,

    pub markets_per_ship: Option<i64>,

    pub mining_waypoints_per_system: Option<i32>,
    pub mining_ships_per_waypoint: Option<i32>,
    pub transport_capacity_per_waypoint: Option<i32>,

    pub trade_mode: Option<database::TradeMode>,
    pub trade_profit_threshold: Option<i32>,

    pub ship_purchase_percentile: Option<f32>,
    pub ship_purchase_stop: Option<bool>,
    pub expand: Option<bool>,
    pub ship_purchase_amount: Option<i32>,

    pub iron_reserve: Option<i64>,
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
pub enum InputFleetConfig {
    Trading(InputTradingConfig),
    Scraping(InputScrapingConfig),
    Mining(InputMiningConfig),
    Charting(InputChartingConfig),
    Construction(InputConstructionConfig),
    Contract(InputContractConfig),
    Manuel(InputManuelConfig),
}

impl InputFleetConfig {
    pub fn into_fleet_config(self) -> database::FleetConfig {
        match self {
            InputFleetConfig::Trading(cfg) => {
                database::FleetConfig::Trading(database::TradingFleetConfig {
                    market_blacklist: cfg.market_blacklist.unwrap_or_default(),
                    market_prefer_list: cfg.market_prefer_list.unwrap_or_default(),
                    purchase_multiplier: cfg.purchase_multiplier.unwrap_or(1.0),
                    ship_market_ratio: cfg.ship_market_ratio.unwrap_or(1.0),
                    min_cargo_space: cfg.min_cargo_space.unwrap_or(0),
                    trade_mode: cfg.trade_mode.unwrap_or(database::TradeMode::ProfitPerHour),
                    trade_profit_threshold: cfg.trade_profit_threshold.unwrap_or(0),
                })
            }
            InputFleetConfig::Scraping(cfg) => {
                database::FleetConfig::Scraping(database::ScrapingFleetConfig {
                    ship_market_ratio: cfg.ship_market_ratio.unwrap_or(1.0),
                    allowed_requests: cfg.allowed_requests.unwrap_or(0),
                    notify_on_shipyard: cfg.notify_on_shipyard.unwrap_or(false),
                })
            }
            InputFleetConfig::Mining(cfg) => {
                database::FleetConfig::Mining(database::MiningFleetConfig {
                    mining_eject_list: cfg.mining_eject_list.unwrap_or_default(),
                    mining_prefer_list: cfg.mining_prefer_list.unwrap_or_default(),
                    ignore_engineered_asteroids: cfg.ignore_engineered_asteroids.unwrap_or(false),
                    stop_all_unstable: cfg.stop_all_unstable.unwrap_or(false),
                    unstable_since_timeout: cfg.unstable_since_timeout.unwrap_or(0),
                    mining_waypoints: cfg.mining_waypoints.unwrap_or(0),
                    syphon_waypoints: cfg.syphon_waypoints.unwrap_or(0),
                    miners_per_waypoint: cfg.miners_per_waypoint.unwrap_or(0),
                    siphoners_per_waypoint: cfg.siphoners_per_waypoint.unwrap_or(0),
                    surveyers_per_waypoint: cfg.surveyers_per_waypoint.unwrap_or(0),
                    mining_transporters_per_waypoint: cfg
                        .mining_transporters_per_waypoint
                        .unwrap_or(0),
                    min_transporter_cargo_space: cfg.min_transporter_cargo_space.unwrap_or(0),
                    min_mining_cargo_space: cfg.min_mining_cargo_space.unwrap_or(0),
                    min_siphon_cargo_space: cfg.min_siphon_cargo_space.unwrap_or(0),
                })
            }
            InputFleetConfig::Charting(cfg) => {
                database::FleetConfig::Charting(database::ChartingFleetConfig {
                    charting_probe_count: cfg.charting_probe_count.unwrap_or(0),
                })
            }
            InputFleetConfig::Construction(cfg) => {
                database::FleetConfig::Construction(database::ConstructionFleetConfig {
                    construction_ship_count: cfg.construction_ship_count.unwrap_or(0),
                    construction_waypoint: cfg.construction_waypoint.unwrap_or_default(),
                })
            }
            InputFleetConfig::Contract(cfg) => {
                database::FleetConfig::Contract(database::ContractFleetConfig {
                    contract_ship_count: cfg.contract_ship_count.unwrap_or(0),
                })
            }
            InputFleetConfig::Manuel(_cfg) => {
                // ManuelConfig is not exported, return default
                database::FleetConfig::Manuel(Default::default())
            }
        }
    }
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct InputTradingConfig {
    pub market_blacklist: Option<Vec<models::TradeSymbol>>,
    pub market_prefer_list: Option<Vec<models::TradeSymbol>>,
    pub purchase_multiplier: Option<f64>,
    pub ship_market_ratio: Option<f64>,
    pub min_cargo_space: Option<i32>,
    pub trade_mode: Option<database::TradeMode>,
    pub trade_profit_threshold: Option<i32>,
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct InputScrapingConfig {
    pub ship_market_ratio: Option<f64>,
    pub allowed_requests: Option<i32>,
    pub notify_on_shipyard: Option<bool>,
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct InputMiningConfig {
    pub mining_eject_list: Option<Vec<models::TradeSymbol>>,
    pub mining_prefer_list: Option<Vec<models::TradeSymbol>>,
    pub ignore_engineered_asteroids: Option<bool>,
    pub stop_all_unstable: Option<bool>,
    pub unstable_since_timeout: Option<i32>,
    pub mining_waypoints: Option<i32>,
    pub syphon_waypoints: Option<i32>,
    pub miners_per_waypoint: Option<i32>,
    pub siphoners_per_waypoint: Option<i32>,
    pub surveyers_per_waypoint: Option<i32>,
    pub mining_transporters_per_waypoint: Option<i32>,
    pub min_transporter_cargo_space: Option<i32>,
    pub min_mining_cargo_space: Option<i32>,
    pub min_siphon_cargo_space: Option<i32>,
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct InputChartingConfig {
    pub charting_probe_count: Option<i32>,
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct InputConstructionConfig {
    pub construction_ship_count: Option<i32>,
    pub construction_waypoint: Option<String>,
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct InputContractConfig {
    pub contract_ship_count: Option<i32>,
}

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct InputManuelConfig {
    pub config: Option<String>,
}
