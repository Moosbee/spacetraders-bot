use std::fmt;

use crate::sql::{self, TradeRoute};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinTradeRoute {
    pub symbol: space_traders_client::models::TradeSymbol,
    pub purchase_wp_symbol: String,
    pub sell_wp_symbol: String,
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
        Some(self.cmp(other))
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

#[derive(Debug, Clone, PartialEq)]
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

    pub ship_symbol: String,

    pub trip_fuel_cost: i32,
    pub trip_fuel_units: i32,
    pub trip_time: chrono::TimeDelta,
    pub trip_units: i32,
    pub trip_total_cost: i32,
    pub trip_total_profit: i32,

    pub trips_per_hour: f32,
    pub profit_per_hour: i32,
}

impl From<PossibleTradeRoute> for MinTradeRoute {
    fn from(value: PossibleTradeRoute) -> Self {
        MinTradeRoute {
            symbol: value.symbol,
            purchase_wp_symbol: value.purchase_wp_symbol.clone(),
            sell_wp_symbol: value.sell_wp_symbol.clone(),
        }
    }
}

impl From<ConcreteTradeRoute> for MinTradeRoute {
    fn from(value: ConcreteTradeRoute) -> Self {
        MinTradeRoute {
            symbol: value.symbol,
            purchase_wp_symbol: value.purchase_wp_symbol.clone(),
            sell_wp_symbol: value.sell_wp_symbol.clone(),
        }
    }
}

impl From<sql::TradeRoute> for MinTradeRoute {
    fn from(value: sql::TradeRoute) -> Self {
        MinTradeRoute {
            symbol: value.symbol,
            purchase_wp_symbol: value.purchase_waypoint,
            sell_wp_symbol: value.sell_waypoint,
        }
    }
}

impl From<ConcreteTradeRoute> for sql::TradeRoute {
    fn from(value: ConcreteTradeRoute) -> Self {
        TradeRoute {
            symbol: value.symbol,
            finished: false,
            ship_symbol: value.ship_symbol.clone(),
            predicted_purchase_price: value.purchase_price,
            predicted_sell_price: value.sell_price,
            purchase_waypoint: value.purchase_wp_symbol.clone(),
            sell_waypoint: value.sell_wp_symbol.clone(),
            ..TradeRoute::default()
        }
    }
}

// impl Ord for ConcreteTradeRoute {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.profit_per_hour.cmp(&other.profit_per_hour)
//     }
// }

impl PartialOrd for ConcreteTradeRoute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Some(self.cmp(other))
        Some(self.profit_per_hour.cmp(&other.profit_per_hour))
    }
}

impl fmt::Display for ConcreteTradeRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}: {} -> {} {}",
            self.ship_symbol,
            self.symbol,
            self.purchase_wp_symbol,
            self.sell_wp_symbol,
            self.profit_per_hour
        )
    }
}

// trait RoutesTrackkeeper {}
