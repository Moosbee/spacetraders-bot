use std::collections::HashSet;

use space_traders_client::models::TradeSymbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinTradeRoute {
    pub symbol: TradeSymbol,
    pub purchase_wp_symbol: String,
    pub sell_wp_symbol: String,
}

impl From<database::TradeRoute> for MinTradeRoute {
    fn from(route: database::TradeRoute) -> Self {
        MinTradeRoute {
            symbol: route.symbol,
            purchase_wp_symbol: route.purchase_waypoint,
            sell_wp_symbol: route.sell_waypoint,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, async_graphql::SimpleObject)]
pub struct RouteLock {
    symbol: TradeSymbol,
    wp_symbol: String,
    is_end: bool,
}

#[derive(Debug, Default, Clone)]
pub struct RoutesTracker {
    routes: HashSet<RouteLock>,
}

#[allow(dead_code)]
impl RoutesTracker {
    pub fn lock(&mut self, route: &MinTradeRoute) -> bool {
        let start: RouteLock = RouteLock {
            symbol: route.symbol,
            wp_symbol: route.purchase_wp_symbol.clone(),
            is_end: false,
        };
        let end: RouteLock = RouteLock {
            symbol: route.symbol,
            wp_symbol: route.sell_wp_symbol.clone(),
            is_end: true,
        };

        let start_val = self.routes.contains(&start);
        let end_val = self.routes.contains(&end);

        if !start_val && !end_val {
            self.routes.insert(start);
            self.routes.insert(end);
            return true;
        }

        false
    }

    pub fn unlock(&mut self, route: &MinTradeRoute) {
        let start: RouteLock = RouteLock {
            symbol: route.symbol,
            wp_symbol: route.purchase_wp_symbol.clone(),
            is_end: false,
        };
        let end: RouteLock = RouteLock {
            symbol: route.symbol,
            wp_symbol: route.sell_wp_symbol.clone(),
            is_end: true,
        };

        self.routes.take(&start);
        self.routes.take(&end);
    }

    pub fn is_locked(&self, route: &MinTradeRoute) -> bool {
        self.is_start_locked(route) || self.is_end_locked(route)
    }

    pub fn is_real_locked(&self, route: &MinTradeRoute) -> bool {
        self.is_start_locked(route) && self.is_end_locked(route)
    }

    fn is_start_locked(&self, route: &MinTradeRoute) -> bool {
        let start: RouteLock = RouteLock {
            symbol: route.symbol,
            wp_symbol: route.purchase_wp_symbol.clone(),
            is_end: false,
        };
        self.routes.contains(&start)
    }

    fn is_end_locked(&self, route: &MinTradeRoute) -> bool {
        let end: RouteLock = RouteLock {
            symbol: route.symbol,
            wp_symbol: route.sell_wp_symbol.clone(),
            is_end: true,
        };
        self.routes.contains(&end)
    }

    pub fn get_locked_routes(&self) -> HashSet<RouteLock> {
        self.routes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(symbol: TradeSymbol, purchase_wp: &str, sell_wp: &str) -> MinTradeRoute {
        MinTradeRoute {
            symbol,
            purchase_wp_symbol: purchase_wp.to_string(),
            sell_wp_symbol: sell_wp.to_string(),
        }
    }

    #[test]
    fn lock_returns_true_only_while_free() {
        let mut tracker = RoutesTracker::default();
        let route = route(TradeSymbol::PreciousStones, "purchase-a", "sell-a");

        assert!(tracker.lock(&route));
        assert!(!tracker.lock(&route));
    }

    #[test]
    fn unlock_allows_relocking() {
        let mut tracker = RoutesTracker::default();
        let route = route(TradeSymbol::PreciousStones, "purchase-a", "sell-a");

        assert!(tracker.lock(&route));
        tracker.unlock(&route);
        assert!(tracker.lock(&route));
    }

    #[test]
    fn is_locked_and_is_real_locked_reflect_state() {
        let mut tracker = RoutesTracker::default();
        let route = route(TradeSymbol::PreciousStones, "purchase-a", "sell-a");

        assert!(!tracker.is_locked(&route));
        assert!(!tracker.is_real_locked(&route));

        assert!(tracker.lock(&route));
        assert!(tracker.is_locked(&route));
        assert!(tracker.is_real_locked(&route));

        tracker.unlock(&route);
        assert!(!tracker.is_locked(&route));
        assert!(!tracker.is_real_locked(&route));
    }

    #[test]
    fn routes_sharing_sell_waypoint_conflict() {
        let mut tracker = RoutesTracker::default();
        let route_a = route(TradeSymbol::PreciousStones, "purchase-a", "sell-shared");
        let route_b = route(TradeSymbol::PreciousStones, "purchase-b", "sell-shared");

        assert!(tracker.lock(&route_a));
        assert!(!tracker.lock(&route_b));
    }

    #[test]
    fn routes_sharing_purchase_waypoint_conflict() {
        let mut tracker = RoutesTracker::default();
        let route_a = route(TradeSymbol::PreciousStones, "purchase-shared", "sell-a");
        let route_b = route(TradeSymbol::PreciousStones, "purchase-shared", "sell-b");

        assert!(tracker.lock(&route_a));
        assert!(!tracker.lock(&route_b));
    }

    #[test]
    fn different_symbols_do_not_conflict() {
        let mut tracker = RoutesTracker::default();
        let route_a = route(
            TradeSymbol::PreciousStones,
            "purchase-shared",
            "sell-shared",
        );
        let route_b = route(TradeSymbol::QuartzSand, "purchase-shared", "sell-shared");

        assert!(tracker.lock(&route_a));
        assert!(tracker.lock(&route_b));
    }

    #[test]
    fn unlocking_frees_shared_waypoint() {
        let mut tracker = RoutesTracker::default();
        let route_a = route(TradeSymbol::PreciousStones, "purchase-a", "sell-shared");
        let route_b = route(TradeSymbol::PreciousStones, "purchase-b", "sell-shared");

        assert!(tracker.lock(&route_a));
        assert!(!tracker.lock(&route_b));

        tracker.unlock(&route_a);
        assert!(tracker.lock(&route_b));
    }

    #[test]
    fn get_locked_routes_reflects_locks() {
        let mut tracker = RoutesTracker::default();
        let route = route(TradeSymbol::PreciousStones, "purchase-a", "sell-a");

        assert!(tracker.lock(&route));

        let locked: HashSet<RouteLock> = tracker.get_locked_routes();
        assert!(locked.contains(&RouteLock {
            symbol: TradeSymbol::PreciousStones,
            wp_symbol: "purchase-a".to_string(),
            is_end: false
        }));
        assert!(locked.contains(&RouteLock {
            symbol: TradeSymbol::PreciousStones,
            wp_symbol: "sell-a".to_string(),
            is_end: true
        }));
    }

    #[test]
    fn converts_from_database_trade_route() {
        let db_route = database::TradeRoute {
            id: 1,
            symbol: TradeSymbol::PreciousStones,
            ship_symbol: "ship-a".to_string(),
            purchase_waypoint: "purchase-a".to_string(),
            sell_waypoint: "sell-a".to_string(),
            status: database::ShipmentStatus::InTransit,
            trade_volume: 10,
            purchase_trade_good_id: None,
            sell_trade_good_id: None,
            estimated_fuel: None,
            trade_mode: database::TradeMode::ProfitPerHour,
            reserved_fund: None,
            fleet_id: None,
            assignment_id: None,
            created_at: chrono::Utc::now(),
        };

        let min_route: MinTradeRoute = db_route.into();

        assert_eq!(min_route.symbol, TradeSymbol::PreciousStones);
        assert_eq!(min_route.purchase_wp_symbol, "purchase-a");
        assert_eq!(min_route.sell_wp_symbol, "sell-a");
    }
}
