use std::sync::Arc;

use database::DbPool;
use serde::{Deserialize, Serialize};
use ship::ShipManager;
use ship::status::ShipStatus;
use space_traders_client::models;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::manager::budget_manager::BudgetManager;
use crate::manager::chart_manager::ChartManagerMessanger;
use crate::manager::construction_manager::ConstructionManagerMessanger;
use crate::manager::contract_manager::ContractManagerMessanger;
use crate::manager::fleet_manager::FleetManagerMessanger;
use crate::manager::fleet_manager::ShipProcurementMessanger;
use crate::manager::mining_manager::MiningManagerMessanger;
use crate::manager::scrapping_manager::ScrappingManagerMessanger;
use crate::manager::ship_task::ShipTaskMessanger;
use crate::manager::trade_manager::TradeManagerMessanger;
#[derive(Debug, Clone)]
pub struct ConductorContext {
    pub api: space_traders_client::Api,
    pub database_pool: DbPool,
    pub ship_manager: Arc<ShipManager<ShipStatus>>,
    pub ship_tasks: ShipTaskMessanger,
    pub construction_manager: ConstructionManagerMessanger,
    pub contract_manager: ContractManagerMessanger,
    pub mining_manager: MiningManagerMessanger,
    pub scrapping_manager: ScrappingManagerMessanger,
    pub trade_manager: TradeManagerMessanger,
    pub fleet_manager: FleetManagerMessanger,
    pub ship_procurement_manager: ShipProcurementMessanger,
    pub chart_manager: ChartManagerMessanger,
    pub budget_manager: Arc<BudgetManager>,
    pub run_info: Arc<RwLock<RunInfo>>,
    pub config: Arc<RwLock<Config>>,
    pub cancellation_tokens: Arc<CancellationTokens>,
}

#[derive(Debug, Clone)]
pub struct CancellationTokens {
    pub global_cancel_token: CancellationToken,
    pub run_cancel_token: CancellationToken,
    pub fast_manager_cancel_token: CancellationToken,
    pub slow_manager_cancel_token: CancellationToken,
    pub fast_ship_cancel_token: CancellationToken,
    pub slow_ship_cancel_token: CancellationToken,
}

#[derive(Deserialize, Serialize, Debug, Clone, async_graphql::SimpleObject)]
pub struct ChannelInfo {
    pub state: ChannelState,
    pub total_capacity: usize,
    pub used_capacity: usize,
    pub free_capacity: usize,
}

#[derive(
    Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, async_graphql::Enum,
)]
pub enum ChannelState {
    Open,
    Closed,
}

#[derive(Deserialize, Serialize, Debug, Clone, async_graphql::SimpleObject)]
pub struct Config {
    pub control_start_sleep: u64,
    pub control_active: bool,

    pub scrapper_start_sleep: u64,
    pub scrap_agents: bool,
    pub update_all_systems: bool,

    pub max_miners_per_waypoint: u32,
    pub mining_eject_list: Vec<models::TradeSymbol>,
    pub mining_prefer_list: Vec<models::TradeSymbol>,
    pub ignore_engineered_asteroids: bool,
    pub unstable_since_timeout: i64, // in seconds
    pub stop_all_unstable: bool,
    pub extra_mining_transporter: i32,

    pub fuel_cost: i32,
    pub antimatter_price: i32,
    pub purchase_multiplier: f32,

    pub market_blacklist: Vec<models::TradeSymbol>,

    pub default_purchase_price: i32,
    pub default_sell_price: i32,
    pub default_profit: i32,

    pub max_update_interval: i32,

    // Markup and margin percentages (as decimals)
    pub markup_percentage: f32,
    pub margin_percentage: f32,

    pub markets_per_ship: i64,

    pub mining_waypoints_per_system: i32,
    pub mining_ships_per_waypoint: i32,
    pub transport_capacity_per_waypoint: i32,

    pub trade_mode: database::TradeMode,
    pub trade_profit_threshold: i32,

    pub ship_purchase_percentile: f32,
    pub ship_purchase_stop: bool,
    pub expand: bool,
    pub ship_purchase_amount: i32,

    pub use_exploration_fleet: bool,

    pub iron_reserve: i64,
}
impl Default for Config {
    fn default() -> Config {
        Config {
            control_start_sleep: 0,
            control_active: false,
            scrapper_start_sleep: 0,
            scrap_agents: false,
            update_all_systems: false,
            max_miners_per_waypoint: 0,
            mining_eject_list: Vec::new(),
            mining_prefer_list: Vec::new(),
            ignore_engineered_asteroids: false,
            unstable_since_timeout: 0,
            stop_all_unstable: false,
            extra_mining_transporter: 0,
            fuel_cost: 0,
            antimatter_price: 0,
            purchase_multiplier: 0.0,
            market_blacklist: Vec::new(),
            default_purchase_price: 0,
            default_sell_price: 0,
            default_profit: 0,
            max_update_interval: 0,
            markup_percentage: 0.0,
            margin_percentage: 0.0,
            markets_per_ship: 0,
            mining_waypoints_per_system: 0,
            mining_ships_per_waypoint: 0,
            transport_capacity_per_waypoint: 0,
            trade_mode: database::TradeMode::default(),
            trade_profit_threshold: 0,
            ship_purchase_percentile: 0.0,
            ship_purchase_stop: false,
            expand: false,
            ship_purchase_amount: 0,
            iron_reserve: 0,
            use_exploration_fleet: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct RunInfo {
    pub agent_symbol: String,
    pub headquarters: String,
    pub starting_faction: models::FactionSymbol,
    pub reset_date: chrono::DateTime<chrono::Utc>,
    pub next_reset_date: chrono::DateTime<chrono::Utc>,
    pub version: String,
}
impl Default for RunInfo {
    fn default() -> RunInfo {
        RunInfo {
            agent_symbol: "".to_string(),
            headquarters: "".to_string(),
            starting_faction: models::FactionSymbol::default(),
            reset_date: chrono::Utc::now(),
            next_reset_date: chrono::Utc::now(),
            version: "".to_string(),
        }
    }
}
