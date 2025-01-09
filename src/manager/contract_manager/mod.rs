mod contract_manager;

pub use contract_manager::ContractManager;
pub use contract_manager::ContractManagerMessanger;
pub use contract_manager::ContractMessage;
use space_traders_client::models;

#[derive(Debug, Clone)]
pub struct ContractShipment {
    pub contract_id: String,
    pub ship_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub units: i32,
    pub destination_symbol: String,
    pub purchase_symbol: String,
}
