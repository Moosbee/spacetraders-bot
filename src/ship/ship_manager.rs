use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use tokio::sync::RwLock;

use crate::types::{Observer, Subject};

use super::MyShip;

#[derive(Debug)]
pub struct ShipManager {
    locked_ships: DashMap<String, MyShip>,
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
                log::error!("Failed to get ship: {} waiting", symbol);
                self.copy.write().await
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
            locked_ships: DashMap::new(),
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
        me.locked_ships.insert(ship.symbol.clone(), ship);
    }

    pub fn get_ship_clone(&self, symbol: &str) -> Option<MyShip> {
        let map = self.copy.try_read().unwrap();
        map.get(symbol).map(|s| s.clone())
    }

    pub fn get_ships_clone(&self) -> HashMap<String, MyShip> {
        let erg = {
            let map = self.copy.try_read().unwrap();
            map.iter().map(|f| f.1.clone()).collect::<Vec<_>>()
        };
        erg.into_iter().map(|f| (f.symbol.clone(), f)).collect()
    }

    pub fn get_ship(
        &self,
        symbol: &str,
    ) -> Option<dashmap::mapref::one::Ref<'_, std::string::String, MyShip>> {
        self.locked_ships.get(symbol)
    }

    pub fn get_ship_mut(
        &self,
        symbol: &str,
    ) -> Option<dashmap::mapref::one::RefMut<'_, std::string::String, MyShip>> {
        self.locked_ships.get_mut(symbol)
    }
}
