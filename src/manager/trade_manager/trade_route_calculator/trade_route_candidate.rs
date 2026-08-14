use space_traders_client::models;

use crate::manager::trade_manager::routes_tracker::MinTradeRoute;

pub struct TradeRouteCandidate {
    pub symbol: space_traders_client::models::TradeSymbol,
    pub purchase_good: Option<database::MarketTradeGood>,
    pub sell_good: Option<database::MarketTradeGood>,
    pub purchase: database::MarketTrade,
    pub sell: database::MarketTrade,
}
impl TradeRouteCandidate {
    pub(crate) fn is_valid(&self, blacklist: &[models::TradeSymbol]) -> bool {
        self.purchase.waypoint_symbol != self.sell.waypoint_symbol
            && !blacklist.contains(&self.symbol)
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
