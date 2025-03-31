use core::fmt;
use std::cmp::Ordering;

use crate::sql;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinTradeRoute {
    pub symbol: space_traders_client::models::TradeSymbol,
    pub purchase_wp_symbol: String,
    pub sell_wp_symbol: String,
}

impl From<PossibleTradeRoute> for MinTradeRoute {
    fn from(value: PossibleTradeRoute) -> Self {
        MinTradeRoute {
            symbol: value.symbol,
            purchase_wp_symbol: value.purchase.waypoint_symbol,
            sell_wp_symbol: value.sell.waypoint_symbol,
        }
    }
}

impl From<ExtrapolatedTradeRoute> for MinTradeRoute {
    fn from(value: ExtrapolatedTradeRoute) -> Self {
        value.route.into()
    }
}

impl From<ConcreteTradeRoute> for MinTradeRoute {
    fn from(value: ConcreteTradeRoute) -> Self {
        value.route.into()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteData {
    pub min_trade_volume: i32,
    pub max_trade_volume: i32,
    pub purchase_price: i32,
    pub sell_price: i32,
    pub profit: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub struct PossibleTradeRoute {
    pub symbol: space_traders_client::models::TradeSymbol,
    pub purchase_good: Option<sql::MarketTradeGood>,
    pub sell_good: Option<sql::MarketTradeGood>,
    pub purchase: sql::MarketTrade,
    pub sell: sql::MarketTrade,
}

impl Ord for PossibleTradeRoute {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare sell_good Option presence (Some is better than None)
        match (&self.sell_good, &other.sell_good) {
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            _ => {} // Both Some or both None, continue to next comparison
        }

        // Then compare purchase_good Option presence
        match (&self.purchase_good, &other.purchase_good) {
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            _ => {} // Both Some or both None, continue to next comparison
        }

        Ordering::Equal
    }
}

// Implement PartialOrd to be consistent with Ord
impl PartialOrd for PossibleTradeRoute {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for PossibleTradeRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} {} -> {} {}",
            self.symbol,
            self.purchase.waypoint_symbol,
            self.purchase_good.is_some(),
            self.sell.waypoint_symbol,
            self.sell_good.is_some()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtrapolatedTradeRoute {
    pub route: PossibleTradeRoute,
    pub data: RouteData,
}

impl fmt::Display for ExtrapolatedTradeRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} -> {} {}",
            self.route.symbol,
            self.route.purchase.waypoint_symbol,
            self.route.sell.waypoint_symbol,
            self.data.profit
        )
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TripStats {
    pub ship_symbol: String,

    pub fuel_units: i32,
    pub time: f64,
    pub distance: f64,

    pub volume: i32,

    pub fuel_cost: i32,
    pub total_cost: i32,
    pub total_profit: i32,

    pub trips_per_hour: f32,
    pub profit_per_hour: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConcreteTradeRoute {
    pub route: PossibleTradeRoute,

    pub data: RouteData,

    pub trip: TripStats,
}

impl From<ConcreteTradeRoute> for sql::TradeRoute {
    fn from(value: ConcreteTradeRoute) -> Self {
        sql::TradeRoute {
            symbol: value.route.symbol,
            status: sql::ShipmentStatus::InTransit,
            ship_symbol: value.trip.ship_symbol,
            predicted_purchase_price: value.data.purchase_price,
            predicted_sell_price: value.data.sell_price,
            trade_volume: value.data.max_trade_volume,
            purchase_waypoint: value.route.purchase.waypoint_symbol,
            sell_waypoint: value.route.sell.waypoint_symbol,
            ..Default::default()
        }
    }
}

impl PartialOrd for ConcreteTradeRoute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let cmp = self.route.cmp(&other.route).reverse();
        if cmp != Ordering::Equal {
            Some(cmp)
        } else {
            Some(self.trip.profit_per_hour.cmp(&other.trip.profit_per_hour))
        }
    }
}

impl fmt::Display for ConcreteTradeRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}: {} -> {} {}/h",
            self.trip.ship_symbol,
            self.route.symbol,
            self.route.purchase.waypoint_symbol,
            self.route.sell.waypoint_symbol,
            self.trip.profit_per_hour
        )
    }
}
