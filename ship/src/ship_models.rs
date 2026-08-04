use std::{collections::HashSet, fmt::Debug, marker::PhantomData};

use chrono::{DateTime, Utc};
use database::DatabaseConnectorAsync;
use space_traders_client::models::{self, ShipRole};
use utils::{Publisher, Subject};

use crate::{
    cargo::CargoState, error::Result, fuel::FuelState, modules::ModuleState, mounts::MountState,
    my_ship_update::InterShipBroadcaster, status::ShipStatus,
};

use super::ShipManager;

/// Marker type for the mutable (original) state of a RustShip.
/// Only one Mutable instance can exist per ship.
pub struct Mutable;

/// Marker type for the immutable (cloned) state of a RustShip.
/// Immutable ships cannot be mutated — enforced at compile time.
pub struct Immutable;

#[derive(Debug, Default, serde::Serialize, Clone, async_graphql::SimpleObject)]
pub struct Condition {
    pub condition: f64,
    pub integrity: f64,
}

#[derive(Debug, Default, serde::Serialize, Clone, async_graphql::SimpleObject)]
pub struct ConditionState {
    pub engine: Condition,
    pub frame: Condition,
    pub reactor: Condition,
}

pub type MyShip = RustShip<ShipStatus>;
pub type MyShipInfo = RustShip<ShipStatus, Immutable>;

#[derive(serde::Serialize)]
pub struct RustShip<T: Clone + Send + Sync, State: Send + Sync = Mutable> {
    pub status: T,
    pub registration_role: ShipRole,
    pub symbol: String,
    pub display_name: String,
    pub engine_speed: i32,
    pub purchase_id: Option<i64>,
    pub cooldown_expiration: Option<DateTime<Utc>>,
    pub cooldown: Option<i32>,
    // Navigation state
    pub nav: super::nav::NavigationState,
    // Cargo state
    pub cargo: CargoState,
    // Fuel state
    pub fuel: FuelState,
    // Mount state
    pub mounts: MountState,
    // Modules state
    pub modules: ModuleState,
    pub engine: models::ship_engine::Symbol,
    pub reactor: models::ship_reactor::Symbol,
    pub frame: models::ship_frame::Symbol,
    // Conditions
    pub conditions: ConditionState,
    #[serde(skip)]
    pub broadcaster: InterShipBroadcaster,
    #[serde(skip)]
    pub pubsub: Publisher<ShipManager<T>, RustShip<T, Immutable>>,
    #[serde(skip)]
    pub _state: PhantomData<State>,
}

impl<T: Default + Clone + Send + Sync> Default for RustShip<T, Mutable> {
    fn default() -> Self {
        Self {
            status: Default::default(),
            purchase_id: None,
            cooldown: Default::default(),
            pubsub: Publisher::new(),
            broadcaster: Default::default(),
            registration_role: Default::default(),
            symbol: Default::default(),
            display_name: Default::default(),
            engine_speed: Default::default(),
            cooldown_expiration: Default::default(),
            nav: Default::default(),
            cargo: Default::default(),
            fuel: Default::default(),
            mounts: Default::default(),
            modules: Default::default(),
            engine: Default::default(),
            reactor: Default::default(),
            frame: Default::default(),
            conditions: Default::default(),
            _state: PhantomData,
        }
    }
}

impl<T: Clone + Send + Sync> Clone for RustShip<T, Immutable> {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            registration_role: self.registration_role,
            display_name: self.display_name.clone(),
            symbol: self.symbol.clone(),
            purchase_id: self.purchase_id,
            engine_speed: self.engine_speed,
            cooldown_expiration: self.cooldown_expiration,
            cooldown: self.cooldown,
            modules: self.modules.clone(),
            nav: self.nav.clone(),
            cargo: self.cargo.clone(),
            fuel: self.fuel.clone(),
            mounts: self.mounts.clone(),
            conditions: self.conditions.clone(),
            broadcaster: self.broadcaster.clone(),
            pubsub: Publisher::new(),
            engine: self.engine,
            reactor: self.reactor,
            frame: self.frame,
            _state: PhantomData,
        }
    }
}

impl<T: Clone + Send + Sync> Clone for RustShip<T, Mutable> {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            registration_role: self.registration_role,
            display_name: self.display_name.clone(),
            symbol: self.symbol.clone(),
            purchase_id: self.purchase_id,
            engine_speed: self.engine_speed,
            cooldown_expiration: self.cooldown_expiration,
            cooldown: self.cooldown,
            modules: self.modules.clone(),
            nav: self.nav.clone(),
            cargo: self.cargo.clone(),
            fuel: self.fuel.clone(),
            mounts: self.mounts.clone(),
            conditions: self.conditions.clone(),
            broadcaster: self.broadcaster.clone(),
            pubsub: Publisher::new(),
            engine: self.engine,
            reactor: self.reactor,
            frame: self.frame,
            _state: PhantomData,
        }
    }
}

impl<T: Debug + Clone + Send + Sync, State: Send + Sync> Debug for RustShip<T, State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustShip")
            .field("status", &self.status)
            .field("registration_role", &self.registration_role)
            .field("symbol", &self.symbol)
            .field("display_name", &self.display_name)
            .field("engine_speed", &self.engine_speed)
            .field("purchase_id", &self.purchase_id)
            .field("cooldown_expiration", &self.cooldown_expiration)
            .field("nav", &self.nav)
            .field("cargo", &self.cargo)
            .field("fuel", &self.fuel)
            .field("mounts", &self.mounts)
            .field("conditions", &self.conditions)
            .finish_non_exhaustive()
    }
}

impl<T: Clone + Send + Sync, State: Send + Sync> From<&RustShip<T, State>> for database::ShipState {
    fn from(value: &RustShip<T, State>) -> Self {
        Self {
            id: 0,
            symbol: value.symbol.clone(),
            display_name: value.display_name.clone(),
            engine_speed: value.engine_speed,

            engine_condition: value.conditions.engine.condition,
            engine_integrity: value.conditions.engine.integrity,
            frame_condition: value.conditions.frame.condition,
            frame_integrity: value.conditions.frame.integrity,
            reactor_condition: value.conditions.reactor.condition,
            reactor_integrity: value.conditions.reactor.integrity,
            fuel_capacity: value.fuel.capacity,
            fuel_current: value.fuel.current,
            cargo_capacity: value.cargo.capacity,
            cargo_units: value.cargo.units,
            cargo_inventory: sqlx::types::Json(value.cargo.inventory.clone()),
            mounts: value.mounts.mounts.clone(),
            modules: value.modules.modules.clone(),
            reactor_symbol: value.reactor,
            frame_symbol: value.frame,
            engine_symbol: value.engine,
            cooldown_expiration: value.cooldown_expiration,
            cooldown: value.cooldown,
            flight_mode: value.nav.flight_mode.to_string(),
            nav_status: value.nav.status.to_string(),
            system_symbol: value.nav.system_symbol.clone(),
            waypoint_symbol: value.nav.waypoint_symbol.clone(),
            route_arrival: value.nav.route.arrival,
            route_departure: value.nav.route.departure_time,
            route_destination_symbol: value.nav.route.destination_symbol.clone(),
            route_destination_system: value.nav.route.destination_system_symbol.clone(),
            route_origin_symbol: value.nav.route.origin_symbol.clone(),
            route_origin_system: value.nav.route.origin_system_symbol.clone(),
            auto_pilot_arrival: value.nav.auto_pilot.as_ref().map(|t| t.arrival),
            auto_pilot_departure_time: value.nav.auto_pilot.as_ref().map(|t| t.departure_time),
            auto_pilot_destination_symbol: value
                .nav
                .auto_pilot
                .as_ref()
                .map(|t| t.destination_symbol.clone()),
            auto_pilot_destination_system_symbol: value
                .nav
                .auto_pilot
                .as_ref()
                .map(|t| t.destination_system_symbol.clone()),
            auto_pilot_origin_symbol: value
                .nav
                .auto_pilot
                .as_ref()
                .map(|t| t.origin_symbol.clone()),
            auto_pilot_origin_system_symbol: value
                .nav
                .auto_pilot
                .as_ref()
                .map(|t| t.origin_system_symbol.clone()),
            auto_pilot_distance: value.nav.auto_pilot.as_ref().map(|t| t.distance),
            auto_pilot_fuel_cost: value.nav.auto_pilot.as_ref().map(|t| t.fuel_cost),
            auto_pilot_travel_time: value.nav.auto_pilot.as_ref().map(|t| t.travel_time),
            created_at: Utc::now(),
        }
    }
}

// ========== Methods available on ALL states (both Mutable and Immutable) ==========

impl<T: Clone + Send + Sync, State: Send + Sync> RustShip<T, State> {
    /// Snapshot the current ship state to the database. Read-only, works on any state.
    pub async fn snapshot(&self, database_pool: &database::DbPool) -> Result<i64> {
        let state = database::ShipState::from(self);

        let id = database::ShipState::insert_get_id(database_pool, &state).await?;

        Ok(id)
    }

    /// Update ship component info in the database (static-like, doesn't use &self).
    pub async fn update_info_db_shipyard(
        ship: models::ShipyardShip,
        database_pool: &database::DbPool,
    ) -> Result<()> {
        database::EngineInfo::upsert(database_pool, &database::EngineInfo::from(*ship.engine))
            .await?;
        database::FrameInfo::upsert(database_pool, &database::FrameInfo::from(*ship.frame)).await?;
        database::ReactorInfo::upsert(database_pool, &database::ReactorInfo::from(*ship.reactor))
            .await?;

        database::ModuleInfo::insert_bulk(
            database_pool,
            &ship
                .modules
                .into_iter()
                .map(database::ModuleInfo::from)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .await?;
        database::MountInfo::insert_bulk(
            database_pool,
            &ship
                .mounts
                .into_iter()
                .map(database::MountInfo::from)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .await?;
        Ok(())
    }

    /// Update ship component info in the database (static-like, doesn't use &self).
    pub async fn update_info_db(
        ship: models::Ship,
        database_pool: &database::DbPool,
    ) -> Result<()> {
        database::EngineInfo::upsert(database_pool, &database::EngineInfo::from(*ship.engine))
            .await?;
        database::FrameInfo::upsert(database_pool, &database::FrameInfo::from(*ship.frame)).await?;
        database::ReactorInfo::upsert(database_pool, &database::ReactorInfo::from(*ship.reactor))
            .await?;

        database::ModuleInfo::insert_bulk(
            database_pool,
            &ship
                .modules
                .into_iter()
                .map(database::ModuleInfo::from)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .await?;
        database::MountInfo::insert_bulk(
            database_pool,
            &ship
                .mounts
                .into_iter()
                .map(database::MountInfo::from)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .await?;
        Ok(())
    }
}

// ========== Methods only available on Immutable (cloned) ships ==========

impl<T: Clone + Send + Sync> RustShip<T, Immutable> {
    /// Convert this immutable snapshot into a mutable (independent) ship.
    /// This is safe because the clone is already detached from the canonical original.
    pub fn into_mutable(self) -> RustShip<T, Mutable> {
        RustShip {
            status: self.status,
            registration_role: self.registration_role,
            symbol: self.symbol,
            display_name: self.display_name,
            engine_speed: self.engine_speed,
            purchase_id: self.purchase_id,
            cooldown_expiration: self.cooldown_expiration,
            cooldown: self.cooldown,
            nav: self.nav,
            cargo: self.cargo,
            fuel: self.fuel,
            mounts: self.mounts,
            modules: self.modules,
            engine: self.engine,
            reactor: self.reactor,
            frame: self.frame,
            conditions: self.conditions,
            broadcaster: self.broadcaster,
            pubsub: self.pubsub,
            _state: PhantomData,
        }
    }
}

// ========== Methods only available on Mutable (the original) ships ==========

impl<T: Clone + Send + Sync> RustShip<T, Mutable> {
    /// Create an immutable snapshot of this ship for read-only use.
    pub fn to_immutable(&self) -> RustShip<T, Immutable> {
        RustShip {
            status: self.status.clone(),
            registration_role: self.registration_role,
            symbol: self.symbol.clone(),
            display_name: self.display_name.clone(),
            engine_speed: self.engine_speed,
            purchase_id: self.purchase_id,
            cooldown_expiration: self.cooldown_expiration,
            cooldown: self.cooldown,
            nav: self.nav.clone(),
            cargo: self.cargo.clone(),
            fuel: self.fuel.clone(),
            mounts: self.mounts.clone(),
            modules: self.modules.clone(),
            engine: self.engine,
            reactor: self.reactor,
            frame: self.frame,
            conditions: self.conditions.clone(),
            broadcaster: self.broadcaster.clone(),
            pubsub: Publisher::new(),
            _state: PhantomData,
        }
    }

    /// Update the ship's state from a server response.
    pub fn update(&mut self, ship: models::Ship) {
        self.symbol = ship.symbol;
        self.engine_speed = ship.engine.speed;
        self.registration_role = ship.registration.role;
        self.update_cooldown(&ship.cooldown);
        self.nav.update(&ship.nav);
        self.cargo.update(&ship.cargo);
        self.fuel.update(&ship.fuel);
        self.reactor = ship.reactor.symbol;
        self.frame = ship.frame.symbol;
        self.engine = ship.engine.symbol;
        self.mounts.update(&ship.mounts);
        self.modules.update(&ship.modules);

        self.conditions.engine.condition = ship.engine.condition;
        self.conditions.engine.integrity = ship.engine.integrity;
        self.conditions.frame.condition = ship.frame.condition;
        self.conditions.frame.integrity = ship.frame.integrity;
        self.conditions.reactor.condition = ship.reactor.condition;
        self.conditions.reactor.integrity = ship.reactor.integrity;
    }

    /// Notify observers of a state change by sending an immutable snapshot.
    pub async fn notify(&self, _loud: bool) {
        self.pubsub.notify_observers(self.to_immutable()).await;
    }

    pub async fn apply_from_db(
        &mut self,
        database_pool: database::DbPool,
    ) -> Result<database::ShipInfo> {
        self.apply_from_db_ship(database_pool, None).await
    }

    pub async fn reload(&mut self, api: &space_traders_client::Api) -> Result<()> {
        let ship = api.get_my_ship(&self.symbol).await?;
        self.update(*ship.data);
        Ok(())
    }

    pub async fn apply_from_db_ship(
        &mut self,
        database_pool: database::DbPool,
        assignment_id: Option<i64>,
    ) -> Result<database::ShipInfo> {
        let db_ship = database::ShipInfo::get_by_id(&database_pool, &self.symbol).await?;
        let ship_info = match db_ship {
            Some(db_ship) => db_ship,
            None => {
                let display_name = if self.display_name.is_empty() {
                    self.symbol.clone()
                } else {
                    self.display_name.clone()
                };
                let ship_info = database::ShipInfo {
                    purchase_id: self.purchase_id,
                    symbol: self.symbol.clone(),
                    display_name,
                    active: true,
                    assignment_id,
                    temp_assignment_id: None,
                };
                database::ShipInfo::upsert(&database_pool, &ship_info).await?;
                ship_info
            }
        };

        self.update_ship_info(ship_info.clone());

        self.notify(true).await;

        Ok(ship_info)
    }

    fn update_ship_info(&mut self, ship_info: database::ShipInfo) {
        self.display_name = ship_info.display_name;
        self.symbol = ship_info.symbol;
        self.purchase_id = ship_info.purchase_id;
    }
}
