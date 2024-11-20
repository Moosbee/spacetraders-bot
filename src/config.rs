use lazy_static::lazy_static;
use serde::Deserialize;
use space_traders_client::models;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub contracts: ContractFleet,
    pub market: MarketScrapers,
    pub trading: TradingFleet,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ContractFleet {
    pub start_sleep_duration: u64,
    pub max_contracts: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MarketScrapers {
    pub start_sleep_duration: u64,
    pub max_scraps: u32,
    pub scrap_interval: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TradingFleet {
    pub start_sleep_duration: u64,
    pub fuel_cost: i32,
    pub purchase_multiplier: f32,
    pub trade_cycle: u32,

    pub blacklist: Vec<models::TradeSymbol>,
}

lazy_static! {
    pub static ref CONFIG: Config =
        serde_json::from_str(&std::fs::read_to_string("config.json").unwrap()).unwrap();
}
