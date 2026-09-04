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
use crate::supply_chain_mapping::SupplyChainMapping;
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
    pub supply_chain_mapping: Arc<SupplyChainMapping>,
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

    pub antimatter_price: i32,

    pub default_purchase_price: i32,
    pub default_sell_price: i32,

    pub max_update_interval: i32,

    pub ship_purchase_percentile: f32,
    pub ship_purchase_stop: bool,
    pub expand: bool,
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
            antimatter_price: 0,
            default_purchase_price: 0,
            default_sell_price: 0,
            max_update_interval: 0,
            ship_purchase_percentile: 0.0,
            ship_purchase_stop: false,
            use_exploration_fleet: false,
            expand: false,
            iron_reserve: 0,
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
    pub total_systems: i32,
    pub total_waypoints: i32,
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
            total_systems: 0,
            total_waypoints: 0,
        }
    }
}
