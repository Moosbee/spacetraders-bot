use std::sync::Weak;
use std::{fmt::Debug, sync::Arc};

use dashmap::DashMap;
use lockable::{AsyncLimit, Lockable, LockableHashMap, SyncLimit};
use space_traders_client::models;
use tokio::sync::RwLock;

use crate::manager::chart_manager::ChartManagerMessanger;
use crate::manager::construction_manager::ConstructionManagerMessanger;
use crate::manager::contract_manager::ContractManagerMessanger;
use crate::manager::mining_manager::MiningManagerMessanger;
use crate::manager::scrapping_manager::ScrappingManagerMessanger;
use crate::manager::ship_task::ShipTaskMessanger;
use crate::manager::trade_manager::TradeManagerMessanger;
use crate::{ship::ShipManager, sql::DbPool};

/// Trait representing an observer that can be updated
pub trait Observer<K> {
    /// Asynchronous method to update the observer with new data
    async fn update(&self, data: K);
}

/// Trait representing a subject that can manage and notify observers
pub trait Subject<T: Observer<K>, K> {
    /// Register a new observer
    fn register_observer(&mut self, observer: Weak<T>);

    /// Remove a specific observer
    fn remove_observer(&mut self, observer: &T);

    /// Notify all registered observers with new data
    async fn notify_observers(&self, data: K);
}

/// Generic publisher implementing the Subject trait
pub struct Publisher<T: Observer<K>, K> {
    /// Weak references to observers to prevent strong reference cycles
    observers: Vec<Weak<T>>,

    /// Phantom data to handle the generic type K
    _marker: std::marker::PhantomData<K>,
}

impl<T: Observer<K>, K> Publisher<T, K> {
    /// Create a new Publisher instance
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Observer<K>, K: Clone> Subject<T, K> for Publisher<T, K> {
    fn register_observer(&mut self, observer: Weak<T>) {
        self.observers.push(observer);
    }

    fn remove_observer(&mut self, observer: &T) {
        // Remove the specific observer and clean up any expired weak references
        self.observers.retain(|weak_ref| {
            // Keep references that are either the target observer or still valid
            weak_ref.upgrade().map_or(false, |strong_ref| {
                !std::ptr::eq(strong_ref.as_ref(), observer)
            })
        });
    }

    async fn notify_observers(&self, data: K) {
        // Notify all valid observers
        for weak_observer in &self.observers {
            if let Some(observer) = weak_observer.upgrade() {
                observer.update(data.clone()).await;
            }
        }
    }
}

// Optional: Implement Debug for Publisher if needed
impl<T: Observer<K> + Debug, K: Clone + Debug> Debug for Publisher<T, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Publisher")
            .field("observers_count", &self.observers.len())
            .finish()
    }
}

// Convenience method to create a new Publisher
impl<T: Observer<K>, K> Default for Publisher<T, K> {
    fn default() -> Self {
        Self::new()
    }
}

// Helper method to safely access DashMap
pub fn safely_get_map<'a, K, V>(
    map: &'a DashMap<K, V>,
    key: &K,
) -> Option<dashmap::mapref::one::Ref<'a, K, V>>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    let result = map.try_get(key);
    if result.is_locked() {
        log::warn!("safely_get_map access locked, retrying");
        map.get(key)
    } else {
        result.try_unwrap()
    }
}

// Helper method to safely access DashMap
pub fn safely_get_mut_map<'a, K, V>(
    map: &'a DashMap<K, V>,
    key: &K,
) -> Option<dashmap::mapref::one::RefMut<'a, K, V>>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    let result = map.try_get_mut(key);
    if result.is_locked() {
        log::warn!("safely_get_mut_map access locked, retrying");
        map.get_mut(key)
    } else {
        result.try_unwrap()
    }
}

pub async fn safely_get_lock_mut_map<K, V>(
    map: &LockableHashMap<K, V>,
    key: K,
) -> <LockableHashMap<K, V> as Lockable<K, V>>::Guard<'_>
where
    K: std::hash::Hash + Eq + Clone + Debug,
    V: Clone,
{
    let result = map.try_lock(key.clone(), SyncLimit::no_limit()).unwrap();

    if result.is_none() {
        log::warn!(
            "safely_get_lock_mut_map access locked key: {:?}, retrying line: {}",
            key,
            line!()
        );
        let result = map.async_lock(key.clone(), AsyncLimit::no_limit()).await;
        log::warn!("safely_get_lock_mut_map access {:?} unlocked", key);
        result.unwrap()
    } else {
        result.unwrap()
    }
}

pub trait WaypointCan {
    fn is_marketplace(&self) -> bool;
    fn is_minable(&self) -> bool;
    fn is_sipherable(&self) -> bool;
    fn is_shipyard(&self) -> bool;
    fn is_jump_gate(&self) -> bool;
    fn is_charted(&self) -> bool;
}

impl WaypointCan for space_traders_client::models::Waypoint {
    fn is_marketplace(&self) -> bool {
        self.traits
            .iter()
            .any(|t| t.symbol == space_traders_client::models::WaypointTraitSymbol::Marketplace)
    }

    fn is_minable(&self) -> bool {
        (self.r#type == space_traders_client::models::WaypointType::Asteroid
            || self.r#type == space_traders_client::models::WaypointType::AsteroidField
            || self.r#type == space_traders_client::models::WaypointType::EngineeredAsteroid)
            && !self.traits.iter().any(|t| {
                t.symbol == space_traders_client::models::WaypointTraitSymbol::UnstableComposition
            })
    }

    fn is_sipherable(&self) -> bool {
        self.r#type == space_traders_client::models::WaypointType::GasGiant
    }

    fn is_shipyard(&self) -> bool {
        self.traits
            .iter()
            .any(|t| t.symbol == space_traders_client::models::WaypointTraitSymbol::Shipyard)
    }

    fn is_jump_gate(&self) -> bool {
        self.r#type == space_traders_client::models::WaypointType::JumpGate
    }

    fn is_charted(&self) -> bool {
        self.chart.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct ConductorContext {
    pub api: crate::api::Api,
    pub database_pool: DbPool,
    pub ship_manager: Arc<ShipManager>,
    pub ship_tasks: ShipTaskMessanger,
    pub construction_manager: ConstructionManagerMessanger,
    pub contract_manager: ContractManagerMessanger,
    pub mining_manager: MiningManagerMessanger,
    pub scrapping_manager: ScrappingManagerMessanger,
    pub trade_manager: TradeManagerMessanger,
    pub chart_manager: ChartManagerMessanger,
    pub run_info: Arc<RwLock<RunInfo>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RunInfo {
    pub agent_symbol: String,
    pub headquarters: String,
    pub starting_faction: models::FactionSymbol,
    pub reset_date: chrono::NaiveDate,
    pub next_reset_date: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

pub trait SendFuture: core::future::Future {
    fn send(self) -> impl core::future::Future<Output = Self::Output> + Send
    where
        Self: Sized + Send,
    {
        self
    }
}
impl<T: core::future::Future> SendFuture for T {}
