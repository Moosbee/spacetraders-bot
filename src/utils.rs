use std::sync::Arc;

use database::DbPool;
use ship::ShipManager;
use tokio::sync::RwLock;
use utils::RunInfo;

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
    pub ship_manager: Arc<ShipManager>,
    pub ship_tasks: ShipTaskMessanger,
    pub construction_manager: ConstructionManagerMessanger,
    pub contract_manager: ContractManagerMessanger,
    pub mining_manager: MiningManagerMessanger,
    pub scrapping_manager: ScrappingManagerMessanger,
    pub trade_manager: TradeManagerMessanger,
    pub fleet_manager: FleetManagerMessanger,
    pub chart_manager: ChartManagerMessanger,
    pub run_info: Arc<RwLock<RunInfo>>,
}
