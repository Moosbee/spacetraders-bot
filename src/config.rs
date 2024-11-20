use lazy_static::lazy_static;
use serde::Deserialize;
use space_traders_client::models;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub contracts: ContractFleet,
    pub market: MarketScrapers,
    pub trading: TradingFleet,
    pub construction: ConstructionFleet,
    pub mining: MiningFleet,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ContractFleet {
    pub start_sleep_duration: u64,
    pub max_contracts: i32,
    pub active: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MarketScrapers {
    pub start_sleep_duration: u64,
    pub max_scraps: u32,
    pub scrap_interval: u64,
    pub active: bool,
    pub agents: bool,
    pub agent_interval: u64,
    pub max_agent_scraps: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TradingFleet {
    pub start_sleep_duration: u64,
    pub fuel_cost: i32,
    pub purchase_multiplier: f32,
    pub trade_cycle: u32,

    pub blacklist: Vec<models::TradeSymbol>,
    pub active: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConstructionFleet {
    pub start_sleep_duration: u64,
    pub active: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MiningFleet {
    pub start_sleep_duration: u64,
    pub active: bool,
}

lazy_static! {
    pub static ref CONFIG: Config =
        serde_json::from_str(&std::fs::read_to_string("config.json").unwrap()).unwrap();
}
