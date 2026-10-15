use space_traders_client::models;

use crate::manager::trade_manager::routes_tracker::MinTradeRoute;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TradeRouteCandidate {
    pub symbol: space_traders_client::models::TradeSymbol,
    pub purchase_good: Option<database::MarketTradeGood>,
    pub sell_good: Option<database::MarketTradeGood>,
    pub purchase: database::MarketTrade,
    pub sell: database::MarketTrade,

    pub purchase_waypoint_market_trades: Vec<database::MarketTrade>,
    pub sell_waypoint_market_trades: Vec<database::MarketTrade>,
}
impl TradeRouteCandidate {
    pub(crate) fn is_valid(&self, blacklist: &[models::TradeSymbol]) -> bool {
        self.purchase.waypoint_symbol != self.sell.waypoint_symbol
            && !blacklist.contains(&self.symbol)
    }

    // indicate if the trade route is definitely unprofitable
    pub(crate) fn def_unprofitable(&self) -> bool {
        if let Some(purchase_good) = &self.purchase_good
            && let Some(sell_good) = &self.sell_good
        {
            if purchase_good.purchase_price > sell_good.sell_price {
                return true;
            }
        }
        false
    }
}

impl From<TradeRouteCandidate> for MinTradeRoute {
    fn from(value: TradeRouteCandidate) -> Self {
        MinTradeRoute {
            symbol: value.symbol,
            purchase_wp_symbol: value.purchase.waypoint_symbol,
            sell_wp_symbol: value.sell.waypoint_symbol,
        }
    }
}
