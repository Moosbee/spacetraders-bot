use core::fmt;
use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;

use crate::sql;

#[derive(Debug, Clone)]
pub struct ConductorContext {
    pub api: crate::api::Api,
    pub database_pool: sqlx::PgPool,
    pub ship_roles: HashMap<String, crate::ship::models::Role>,
    pub my_ships: Arc<DashMap<String, crate::ship::MyShip>>,
    pub all_waypoints:
        Arc<DashMap<String, HashMap<String, space_traders_client::models::Waypoint>>>,
}

pub trait Conductor: Send + Sync {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>>;
    fn get_name(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PossibleTradeRoute {
    pub symbol: space_traders_client::models::TradeSymbol,
    pub export: sql::MarketTradeGood,
    pub import: sql::MarketTradeGood,
    pub min_trade_volume: i32,
    pub max_trade_volume: i32,
    pub purchase_wp_symbol: String,
    pub sell_wp_symbol: String,
    pub purchase_price: i32,
    pub sell_price: i32,
    pub profit: i32,
}

impl PartialOrd for PossibleTradeRoute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.profit.partial_cmp(&other.profit)
    }
}

impl Ord for PossibleTradeRoute {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.profit.cmp(&other.profit)
    }
}

impl fmt::Display for PossibleTradeRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} -> {} {}",
            self.symbol, self.purchase_wp_symbol, self.sell_wp_symbol, self.profit
        )
    }
}

pub struct ConcreteTradeRoute {
    pub symbol: space_traders_client::models::TradeSymbol,
    pub export: sql::MarketTradeGood,
    pub import: sql::MarketTradeGood,
    pub min_trade_volume: i32,
    pub max_trade_volume: i32,
    pub purchase_wp_symbol: String,
    pub sell_wp_symbol: String,
    pub purchase_price: i32,
    pub sell_price: i32,
    pub profit: i32,
}
