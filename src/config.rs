use std::net::SocketAddr;

use lazy_static::lazy_static;
use serde::Deserialize;
use space_traders_client::models;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub symbol: String,
    pub contracts: ContractFleet,
    pub market: MarketScrapers,
    pub trading: TradingFleet,
    pub construction: ConstructionFleet,
    pub mining: MiningFleet,
    pub control_server: ControlServer,
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

    pub default_purchase_price: i32,
    pub default_sell_price: i32,
    pub default_profit: i32,

    // Markup and margin percentages (as decimals)
    pub markup_percentage: f32,
    pub margin_percentage: f32,
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
    pub max_miners_per_waypoint: u32,
    pub max_extractions_per_miner: u32,
    pub blacklist: Vec<models::TradeSymbol>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ControlServer {
    pub socket_address: SocketAddr,
    pub start_sleep_duration: u64,
    pub active: bool,
}

lazy_static! {
    pub static ref CONFIG: Config =
        serde_json::from_str(&std::fs::read_to_string("config.json").unwrap()).unwrap();
}
