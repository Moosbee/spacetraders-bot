use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, MutexGuard},
};

use space_traders_client::{ShipSymbol, WaypointSymbol};
use tokio::sync::Notify;

use crate::error::{Error, Result};

#[derive(Debug, Default)]
struct Inner {
    /// waypoint -> ships currently present there
    ships: HashMap<WaypointSymbol, HashSet<ShipSymbol>>,
    /// waypoint -> number of live reservations
    reservations: HashMap<WaypointSymbol, usize>,
}

impl Inner {
    fn ship_count(&self, wp: &WaypointSymbol) -> usize {
        self.ships.get(wp).map_or(0, HashSet::len)
    }

    fn reservation_count(&self, wp: &WaypointSymbol) -> usize {
        self.reservations.get(wp).copied().unwrap_or(0)
    }

    fn add_reservation(&mut self, wp: &WaypointSymbol) {
        *self.reservations.entry(wp.to_owned()).or_insert(0) += 1;
    }

    fn remove_reservation(&mut self, wp: &WaypointSymbol) {
        if let Some(n) = self.reservations.get_mut(wp) {
            *n -= 1;
            if *n == 0 {
                self.reservations.remove(wp);
            }
        }
    }

    /// A ship may leave if nobody relies on this waypoint, or if another
    /// ship is there to keep it occupied.
    fn can_leave(&self, wp: &WaypointSymbol) -> bool {
        self.reservation_count(wp) == 0 || self.ship_count(wp) >= 2
    }
}

#[derive(Debug)]
pub struct ShipyardShipReservation {
    inner: Mutex<Inner>,
    notify: Notify,
}

/// Keeps at least one ship at the waypoint until dropped.
/// Multiple reservations may exist for the same waypoint.
#[derive(Debug)]
pub struct WaypointReservation<'a> {
    yard: &'a ShipyardShipReservation,
    wp: WaypointSymbol,
}

impl WaypointReservation<'_> {
    pub fn waypoint(&self) -> &WaypointSymbol {
        &self.wp
    }
}

impl Clone for WaypointReservation<'_> {
    fn clone(&self) -> Self {
        // A clone is an independent reservation of the same waypoint.
        self.yard.lock().add_reservation(&self.wp);
        Self {
            yard: self.yard,
            wp: self.wp.clone(),
        }
    }
}

impl Drop for WaypointReservation<'_> {
    fn drop(&mut self) {
        self.yard.lock().remove_reservation(&self.wp);
        // Ships waiting in `deregister` may now be free to leave.
        self.yard.notify.notify_waiters();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ShipState {
    /// The ship is at the waypoint and free to leave.
    Present,
    /// The ship is at the waypoint and pinned: it is the only ship there
    /// and a reservation exists, so it can't deregister yet.
    Reserved,
}

impl Default for ShipyardShipReservation {
    fn default() -> Self {
        Self::new()
    }
}

impl ShipyardShipReservation {
    // new workflow
    // the scrapper arrives at the shipyard
    // registers himself as present
    // waits until the scrap and scraps
    // the ship will try to unregister himself, but has to wait for the reservation to release

    // the manager will reserve one, multiple or all waypoints
    // while the manager has a reservation for a waypoint the waypoint must have at least one ship

    // if a ship tries to unregister itself, it has to wait for the reservation to release or until another ship also registers itself to the same waypoint

    // there may be multiple WaypointReservations for a single waypoint

    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::default()),
            notify: Notify::new(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        // The state is always left consistent, so recover from poisoning.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Registers the ship as present at the waypoint (idempotent).
    pub async fn register(&self, ship: &ShipSymbol, wp: &WaypointSymbol) -> Result<()> {
        self.lock()
            .ships
            .entry(wp.to_owned())
            .or_default()
            .insert(ship.to_owned());
        // Wakes `wait_for_ships` and any ship waiting to deregister
        // (a second ship at the waypoint releases it).
        self.notify.notify_waiters();
        Ok(())
    }

    /// Waits while reserved, then removes the ship.
    pub async fn deregister(&self, ship: &ShipSymbol, wp: &WaypointSymbol) -> Result<ShipSymbol> {
        loop {
            // Create the future *before* checking so no wakeup is missed.
            let notified = self.notify.notified();
            {
                let mut inner = self.lock();
                let present = inner.ships.get(wp).is_some_and(|s| s.contains(ship));
                if !present {
                    return Err(Error::General(format!(
                        "ship {ship} is not present at {wp}"
                    )));
                }
                if inner.can_leave(wp) {
                    // Check and removal happen under one lock, so two ships
                    // can't both leave when only one is allowed to.
                    let set = inner.ships.get_mut(wp).expect("checked above");
                    set.remove(ship);
                    if set.is_empty() {
                        inner.ships.remove(wp);
                    }
                    drop(inner);
                    self.notify.notify_waiters();
                    return Ok(ship.to_owned());
                }
            }
            notified.await;
        }
    }

    pub async fn state(&self, ship: &ShipSymbol, wp: &WaypointSymbol) -> Option<ShipState> {
        let inner = self.lock();
        if !inner.ships.get(wp)?.contains(ship) {
            return None;
        }
        if inner.reservation_count(wp) > 0 && inner.ship_count(wp) == 1 {
            Some(ShipState::Reserved)
        } else {
            Some(ShipState::Present)
        }
    }

    /// Fails if no ship is present at the waypoint; released on drop.
    pub async fn reserve<'a>(&'a self, wp: &WaypointSymbol) -> Result<WaypointReservation<'a>> {
        let mut inner = self.lock();
        if inner.ship_count(wp) == 0 {
            return Err(Error::General(format!("no ship present at {wp}")));
        }
        inner.add_reservation(wp);
        Ok(WaypointReservation {
            yard: self,
            wp: wp.to_owned(),
        })
    }

    /// Reserves every waypoint that currently has a ship, atomically.
    pub async fn reserve_all<'a>(
        &'a self,
    ) -> Result<HashMap<WaypointSymbol, WaypointReservation<'a>>> {
        let mut inner = self.lock();
        let wps: Vec<_> = inner.ships.keys().cloned().collect();
        let mut out = HashMap::with_capacity(wps.len());
        for wp in wps {
            inner.add_reservation(&wp);
            out.insert(wp.clone(), WaypointReservation { yard: self, wp });
        }
        Ok(out)
    }

    /// Waits until at least one ship is present; resolves instantly if one is.
    pub async fn wait_for_ships(&self) -> Result<()> {
        loop {
            let notified = self.notify.notified();
            if !self.lock().ships.is_empty() {
                return Ok(());
            }
            notified.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn sh(sh: &str) -> ShipSymbol {
        ShipSymbol(sh.to_owned())
    }
    fn wp(wp: &str) -> WaypointSymbol {
        WaypointSymbol(wp.to_owned())
    }

    #[tokio::test]
    async fn deregister_waits_for_reservation() {
        let yard = ShipyardShipReservation::new();
        let ship_a = sh("A");
        let wp_x = wp("X");
        yard.register(&ship_a, &wp_x).await.unwrap();
        let res = yard.reserve(&wp_x).await.unwrap();
        assert_eq!(yard.state(&ship_a, &wp_x).await, Some(ShipState::Reserved));

        let dereg = yard.deregister(&ship_a, &wp_x);
        tokio::pin!(dereg);
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut dereg)
                .await
                .is_err()
        );

        drop(res);
        assert_eq!(dereg.await.unwrap(), ship_a);
    }

    #[tokio::test]
    async fn second_ship_releases_first() {
        let yard = ShipyardShipReservation::new();
        yard.register(&sh("A"), &wp("X")).await.unwrap();
        let _res = yard.reserve(&wp("X")).await.unwrap();
        yard.register(&sh("B"), &wp("X")).await.unwrap();
        assert_eq!(yard.deregister(&sh("A"), &wp("X")).await.unwrap(), sh("A"));
        assert_eq!(
            yard.state(&sh("B"), &wp("X")).await,
            Some(ShipState::Reserved)
        );
    }
}
