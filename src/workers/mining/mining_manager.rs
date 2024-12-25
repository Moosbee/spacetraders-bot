use std::{
    collections::{HashMap, HashSet},
    future,
};

use futures::StreamExt;
use lockable::LockableHashMap;
use space_traders_client::models::waypoint;

use crate::{
    config::CONFIG,
    types::{safely_get_lock_mut_map, SendFuture},
};

pub type WaypointInfo = (String, HashSet<String>, chrono::DateTime<chrono::Utc>);

#[derive(Debug)]
pub struct MiningManager {
    mining_places: LockableHashMap<String, WaypointInfo>,
}

impl MiningManager {
    pub fn new() -> MiningManager {
        MiningManager {
            mining_places: LockableHashMap::new(),
        }
    }

    pub async fn assign_to(&self, ship_symbol: &String, waypoint: &waypoint::Waypoint) -> bool {
        let waypoint_symbol = waypoint.symbol.clone();

        let mut waypoint =
            safely_get_lock_mut_map(&self.mining_places, waypoint_symbol.clone()).await;

        let waypoint = waypoint.value_or_insert((
            waypoint_symbol,
            HashSet::new(),
            (chrono::Utc::now() - chrono::Duration::hours(10)),
        ));

        if waypoint.1.contains(ship_symbol) {
            return true;
        }

        if waypoint.1.len() >= CONFIG.mining.max_miners_per_waypoint.try_into().unwrap() {
            return false;
        }

        waypoint.1.insert(ship_symbol.clone());

        true
    }

    #[allow(dead_code)]
    pub async fn unassign_from(&self, ship_symbol: &String, waypoint: &waypoint::Waypoint) -> bool {
        let waypoint_symbol = waypoint.symbol.clone();

        let mut waypoint =
            safely_get_lock_mut_map(&self.mining_places, waypoint_symbol.clone()).await;

        let waypoint = waypoint.value_mut().unwrap();

        waypoint.1.remove(ship_symbol);

        true
    }

    pub async fn up_date(&self, waypoint: &str) {
        let mut waypoint = safely_get_lock_mut_map(&self.mining_places, waypoint.to_string()).await;

        let waypoint = waypoint.value_mut().unwrap();

        waypoint.2 = chrono::Utc::now();
    }

    pub async fn get_count(&self, waypoint: &waypoint::Waypoint) -> usize {
        let waypoint = safely_get_lock_mut_map(&self.mining_places, waypoint.symbol.clone()).await;

        waypoint.value().map(|s| s.1.len()).unwrap_or(0)
    }

    pub async fn get_info(&self, waypoint: &str) -> Option<WaypointInfo> {
        let waypoint = safely_get_lock_mut_map(&self.mining_places, waypoint.to_string()).await;

        let erg = waypoint.value().map(|s| s.clone());

        erg
        // (String::new(), HashSet::new(), chrono::Utc::now())
    }

    pub async fn get_all(&self) -> HashMap<String, WaypointInfo> {
        let map = self.mining_places.lock_all_entries().await;

        let erg = map
            .map(|f| {
                let key = f.key().clone();
                let value = f.value().cloned();

                drop(f);

                (key, value)
            })
            .filter(|f| future::ready(f.1.is_some()))
            .map(|f| (f.0, f.1.unwrap()))
            .collect::<HashMap<_, _>>()
            .send() // https://github.com/rust-lang/rust/issues/96865
            .await;

        erg
    }
}
