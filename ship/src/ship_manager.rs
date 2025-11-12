use std::{collections::HashMap, fmt::Debug, sync::Arc};

use lockable::{AsyncLimit, Lockable, LockableHashMap, SyncLimit};
use tokio::sync::RwLock;
use utils::{Observer, Subject, safely_get_lock_mut_map};

use super::{RustShip, my_ship_update};

#[derive(Debug)]
pub struct ShipManager<T: Clone + Send + Sync + async_graphql::OutputType> {
    locked_ships: LockableHashMap<String, RustShip<T>>,
    copy: RwLock<HashMap<String, RustShip<T>>>,
    mpsc_tx: tokio::sync::broadcast::Sender<RustShip<T>>,
    mpsc_rx: tokio::sync::broadcast::Receiver<RustShip<T>>,
    id: u32,
    broadcaster: my_ship_update::InterShipBroadcaster,
}

// impl Debug for ShipManager {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ShipManager")
//             // .field("locked_ships", &self.locked_ships)
//             // .field("copy", &self.copy)
//             .field("mpsc_tx", &self.mpsc_tx)
//             .field("mpsc_rx", &self.mpsc_rx)
//             .field("id", &self.id)
//             .field("broadcaster", &self.broadcaster)
//             .finish()
//     }
// }

pub type ShipGuard<'a, T> =
    <LockableHashMap<String, RustShip<T>> as Lockable<String, RustShip<T>>>::Guard<'a>;

impl<T: Clone + Send + Sync + async_graphql::OutputType> PartialEq for ShipManager<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Clone + Send + Sync + async_graphql::OutputType> Observer<RustShip<T>> for ShipManager<T> {
    async fn update(&self, data: RustShip<T>) {
        let clone = data.clone();
        let symbol = clone.symbol.clone();
        {
            let map = self.copy.try_write();

            let mut map = match map {
                Ok(m) => m,
                Err(_e) => {
                    tracing::warn!(symbol = %symbol, "Failed to update get ship: waiting");
                    let start = std::time::Instant::now();
                    let map = self.copy.write().await;
                    tracing::warn!(symbol = %symbol, elapsed = ?start.elapsed(), "Got update ship: waiting");
                    map
                }
            };

            map.insert(symbol, clone);
        }
        if let Err(e) = self.mpsc_tx.send(data) {
            tracing::error!(error = %e, "Failed to broadcast ship update");
        }
    }
}

impl<T: Clone + Send + Sync + async_graphql::OutputType> ShipManager<T> {
    pub fn new(broadcaster: my_ship_update::InterShipBroadcaster) -> Self {
        let (mpsc_tx, mpsc_rx) = tokio::sync::broadcast::channel(1000);
        Self {
            locked_ships: LockableHashMap::new(),
            copy: RwLock::new(HashMap::new()),
            mpsc_tx,
            mpsc_rx,
            id: rand::random::<u32>(),
            broadcaster,
        }
    }

    pub fn get_rx(&self) -> tokio::sync::broadcast::Receiver<RustShip<T>> {
        self.mpsc_rx.resubscribe()
    }

    pub async fn add_ship(me: &Arc<ShipManager<T>>, mut ship: RustShip<T>) {
        ship.pubsub.register_observer(Arc::downgrade(me));
        me.copy
            .write()
            .await
            .insert(ship.symbol.clone(), ship.clone());
        let mut guard = me
            .locked_ships
            .async_lock(ship.symbol.clone(), AsyncLimit::no_limit())
            .await
            .unwrap();

        guard.insert(ship);
    }

    pub fn get_clone(&self, symbol: &str) -> Option<RustShip<T>> {
        let map = self.copy.try_read().unwrap();
        map.get(symbol).cloned()
    }

    pub async fn get_all_clone(&self) -> HashMap<String, RustShip<T>> {
        let erg = {
            let map = self.copy.try_read();
            let map = match map {
                Ok(m) => m,
                Err(_) => {
                    tracing::warn!("Failed to get all ships waiting");
                    self.copy.read().await
                }
            };
            map.iter().map(|f| f.1.clone()).collect::<Vec<_>>()
        };
        erg.into_iter().map(|f| (f.symbol.clone(), f)).collect()
    }

    /// Get a mutable reference to a ship by its symbol. The returned value is a lockguard which will be dropped when it's out of scope, releasing the lock on the ship.
    /// If the ship is not found, an error will be returned.
    ///
    /// This function is async because it might wait for other tasks that have locked the ship.
    pub async fn get_mut(&self, symbol: &str) -> ShipGuard<'_, T> {
        // self.locked_ships.get(symbol)
        safely_get_lock_mut_map(&self.locked_ships, symbol.to_owned()).await
    }

    pub async fn try_get_mut(&self, symbol: &str) -> Option<ShipGuard<'_, T>> {
        self.locked_ships
            .try_lock(symbol.to_owned(), SyncLimit::no_limit())
            .unwrap()
    }

    pub fn get_broadcaster(&self) -> my_ship_update::InterShipBroadcaster {
        self.broadcaster.clone()
    }

    pub fn get_ship_count(&self) -> usize {
        self.locked_ships.num_entries_or_locked()
    }
}
