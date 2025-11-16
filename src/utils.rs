use std::net::SocketAddr;
use std::sync::Arc;

use database::DbPool;
use serde::{Deserialize, Serialize};
use ship::status::ShipStatus;
use ship::ShipManager;
use space_traders_client::models;
use tokio::sync::RwLock;

use crate::manager::budget_manager::BudgetManager;
use crate::manager::chart_manager::ChartManagerMessanger;
use crate::manager::construction_manager::ConstructionManagerMessanger;
use crate::manager::contract_manager::ContractManagerMessanger;
use crate::manager::fleet_manager::FleetManagerMessanger;
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
    pub chart_manager: ChartManagerMessanger,
    pub budget_manager: Arc<BudgetManager>,
    pub run_info: Arc<RwLock<RunInfo>>,
    pub config: Arc<RwLock<Config>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub socket_address: SocketAddr,
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

    pub iron_reserve: i64,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct RunInfo {
    pub agent_symbol: String,
    pub headquarters: String,
    pub starting_faction: models::FactionSymbol,
    pub reset_date: chrono::NaiveDate,
    pub next_reset_date: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

#[async_graphql::ComplexObject]
impl RunInfo {
    async fn agent<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<database::Agent, crate::control_api::GraphiQLError> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let agent = database::Agent::get_last_by_symbol(database_pool, &self.agent_symbol)
            .await?
            .ok_or(crate::control_api::GraphiQLError::NotFound)?;
        Ok(agent)
    }
}
