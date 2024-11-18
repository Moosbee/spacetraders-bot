use dashmap::DashSet;
use space_traders_client::models::TradeSymbol;

use super::t_types::MinTradeRoute;

#[derive(Debug, Clone, Default)]
pub struct RoutesTrackKeeper {
    routes: DashSet<(TradeSymbol, String, bool)>,
}

impl RoutesTrackKeeper {
    pub fn lock(&self, route: &MinTradeRoute) -> anyhow::Result<()> {
        let erg = self
            .routes
            .insert((route.symbol, route.purchase_wp_symbol.clone(), false));
        if erg {
            return Err(anyhow::anyhow!("Route is locked"));
        }
        let erg = self
            .routes
            .insert((route.symbol, route.sell_wp_symbol.clone(), true));
        if erg {
            return Err(anyhow::anyhow!("Route is locked"));
        }
        Ok(())
    }

    pub fn unlock(&self, route: &MinTradeRoute) {
        self.routes
            .remove(&(route.symbol, route.purchase_wp_symbol.clone(), false));
        self.routes
            .remove(&(route.symbol, route.sell_wp_symbol.clone(), true));
    }

    pub fn is_locked(&self, route: &MinTradeRoute) -> bool {
        self.is_start_locked(route) || self.is_end_locked(route)
    }

    pub fn is_real_locked(&self, route: &MinTradeRoute) -> bool {
        self.is_start_locked(route) && self.is_end_locked(route)
    }

    fn is_start_locked(&self, route: &MinTradeRoute) -> bool {
        self.routes
            .contains(&(route.symbol, route.purchase_wp_symbol.clone(), false))
    }

    fn is_end_locked(&self, route: &MinTradeRoute) -> bool {
        self.routes
            .contains(&(route.symbol, route.sell_wp_symbol.clone(), true))
    }
}
