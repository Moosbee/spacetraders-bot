use std::collections::HashMap;

use space_traders_client::models::TradeSymbol;

use super::routes::MinTradeRoute;

// pub struct MinTradeRoute {
//     pub symbol: TradeSymbol,
//     pub purchase_wp_symbol: String,
//     pub sell_wp_symbol: String,
// }

type RouteLock = (TradeSymbol, String, bool);

#[derive(Debug, Default)]
pub struct RoutesTracker {
    routes: HashMap<RouteLock, bool>,
}

#[allow(dead_code)]
impl RoutesTracker {
    pub fn lock(&mut self, route: &MinTradeRoute) -> bool {
        let start: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), false);
        let end: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), true);

        let start_val = *self.routes.get(&start).unwrap_or(&false);
        let end_val = *self.routes.get(&end).unwrap_or(&false);

        if !start_val && !end_val {
            self.routes.insert(start, true);
            self.routes.insert(end, true);
            return true;
        }

        false
    }

    pub fn unlock(&mut self, route: &MinTradeRoute) {
        let start: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), false);
        let end: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), true);

        self.routes.insert(start, false);
        self.routes.insert(end, false);
    }

    pub fn is_locked(&self, route: &MinTradeRoute) -> bool {
        self.is_start_locked(route) || self.is_end_locked(route)
    }

    pub fn is_real_locked(&self, route: &MinTradeRoute) -> bool {
        self.is_start_locked(route) && self.is_end_locked(route)
    }

    fn is_start_locked(&self, route: &MinTradeRoute) -> bool {
        let start: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), false);
        *self.routes.get(&start).unwrap_or(&false)
    }

    fn is_end_locked(&self, route: &MinTradeRoute) -> bool {
        let end: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), true);
        *self.routes.get(&end).unwrap_or(&false)
    }
}
