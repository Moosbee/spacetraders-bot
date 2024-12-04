use lockable::{LockableHashMap, SyncLimit};
use log::{debug, warn};
use space_traders_client::models::TradeSymbol;

use super::t_types::MinTradeRoute;

type RouteLock = (TradeSymbol, String, bool);

#[derive(Debug, Default)]
pub struct RoutesTrackKeeper {
    routes: LockableHashMap<RouteLock, bool>,
}

impl RoutesTrackKeeper {
    pub fn lock(&self, route: &MinTradeRoute) -> anyhow::Result<()> {
        let start: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), false);
        let end: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), true);

        let start_lock = self.routes.try_lock(start, SyncLimit::no_limit()).unwrap();
        let end_lock = self.routes.try_lock(end, SyncLimit::no_limit()).unwrap();

        if start_lock.is_none() || end_lock.is_none() {
            warn!("Failed to lock route: {:?}", route);
            return Err(anyhow::anyhow!("Failed to lock route: {:?}", route));
        }

        let mut start_lock = start_lock.unwrap();
        start_lock.insert(true);
        let mut end_lock = end_lock.unwrap();
        end_lock.insert(true);

        drop(start_lock);
        drop(end_lock);

        debug!("Locked route: {:?}", route);
        Ok(())
    }

    pub fn unlock(&self, route: &MinTradeRoute) {
        let start: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), false);
        let end: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), true);

        let start_lock = self.routes.try_lock(start, SyncLimit::no_limit()).unwrap();
        let end_lock = self.routes.try_lock(end, SyncLimit::no_limit()).unwrap();

        if start_lock.is_none() || end_lock.is_none() {
            warn!("Failed to unlock route: {:?}", route);
            return;
        }

        let mut start_lock = start_lock.unwrap();
        start_lock.remove();
        let mut end_lock = end_lock.unwrap();
        end_lock.remove();

        drop(start_lock);
        drop(end_lock);
        debug!("Unlocked route: {:?}", route);
    }

    pub fn is_locked(&self, route: &MinTradeRoute) -> bool {
        self.is_start_locked(route) || self.is_end_locked(route)
    }

    pub fn is_real_locked(&self, route: &MinTradeRoute) -> bool {
        self.is_start_locked(route) && self.is_end_locked(route)
    }

    fn is_start_locked(&self, route: &MinTradeRoute) -> bool {
        let start: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), false);

        let lock = self.routes.try_lock(start, SyncLimit::no_limit()).unwrap();

        // if we couldn't get it it's locked
        let locked = lock.is_none();
        if locked {
            return true;
        }
        let val = lock.as_ref().unwrap().value();
        // if val is empty it's free or when the value is false then it's also free
        let locked = val.is_some() && *val.unwrap();
        drop(lock);
        locked
    }

    fn is_end_locked(&self, route: &MinTradeRoute) -> bool {
        let end: RouteLock = (route.symbol, route.purchase_wp_symbol.clone(), true);

        let lock = self.routes.try_lock(end, SyncLimit::no_limit()).unwrap();

        // if we couldn't get it it's locked
        let locked = lock.is_none();
        if locked {
            return true;
        }
        let val = lock.as_ref().unwrap().value();
        // if val is empty it's free or when the value is false then it's also free
        let locked = val.is_some() && *val.unwrap();
        drop(lock);
        locked
    }
}
