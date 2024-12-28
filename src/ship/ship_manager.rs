use std::{collections::HashMap, sync::Arc};

use lockable::{AsyncLimit, Lockable, LockableHashMap, SyncLimit};
use tokio::sync::RwLock;

use crate::types::{safely_get_lock_mut_map, Observer, Subject};

use super::MyShip;

#[derive(Debug)]
pub struct ShipManager {
    locked_ships: LockableHashMap<String, MyShip>,
    copy: RwLock<HashMap<String, MyShip>>,
    mpsc_tx: tokio::sync::broadcast::Sender<MyShip>,
    mpsc_rx: tokio::sync::broadcast::Receiver<MyShip>,
    id: u32,
}

impl PartialEq for ShipManager {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Observer<MyShip> for ShipManager {
    async fn update(&self, data: MyShip) {
        let clone = data.clone();
        let symbol = clone.symbol.clone();
        {
            let map = self.copy.try_write();

            let mut map = if map.is_err() {
                log::warn!("Failed to update get ship: {} waiting", symbol);
                let map = self.copy.write().await;
                log::warn!("Got update ship: {} waiting", symbol);
                map
            } else {
                map.unwrap()
            };

            map.insert(symbol, clone);
        }
        if let Err(e) = self.mpsc_tx.send(data) {
            log::error!("Failed to broadcast ship update: {}", e);
        }
    }
}

impl ShipManager {
    pub fn new() -> Self {
        let (mpsc_tx, mpsc_rx) = tokio::sync::broadcast::channel(100);
        Self {
            locked_ships: LockableHashMap::new(),
            copy: RwLock::new(HashMap::new()),
            mpsc_tx,
            mpsc_rx,
            id: rand::random::<u32>(),
        }
    }

    pub fn get_rx(&self) -> tokio::sync::broadcast::Receiver<MyShip> {
        self.mpsc_rx.resubscribe()
    }

    pub async fn add_ship(me: &Arc<ShipManager>, mut ship: MyShip) {
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

    pub fn get_clone(&self, symbol: &str) -> Option<MyShip> {
        let map = self.copy.try_read().unwrap();
        map.get(symbol).map(|s| s.clone())
    }

    pub fn get_all_clone(&self) -> HashMap<String, MyShip> {
        let erg = {
            let map = self.copy.try_read();
            let map = if map.is_err() {
                log::warn!("Failed to get all ships waiting");
                self.copy.blocking_read()
            } else {
                map.unwrap()
            };
            map.iter().map(|f| f.1.clone()).collect::<Vec<_>>()
        };
        erg.into_iter().map(|f| (f.symbol.clone(), f)).collect()
    }

    /// Get a mutable reference to a ship by its symbol. The returned value is a lockguard which will be dropped when it's out of scope, releasing the lock on the ship.
    /// If the ship is not found, an error will be returned.
    ///
    /// This function is async because it might wait for other tasks that have locked the ship.
    pub async fn get_mut(
        &self,
        symbol: &str,
    ) -> <LockableHashMap<String, MyShip> as Lockable<String, MyShip>>::Guard<'_> {
        let erg = safely_get_lock_mut_map(&self.locked_ships, symbol.to_owned()).await;
        // self.locked_ships.get(symbol)
        erg
    }

    pub async fn try_get_mut(
        &self,
        symbol: &str,
    ) -> Option<<LockableHashMap<String, MyShip> as Lockable<String, MyShip>>::Guard<'_>> {
        let erg = self
            .locked_ships
            .try_lock(symbol.to_owned(), SyncLimit::no_limit())
            .unwrap();

        erg
    }
}
