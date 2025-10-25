use std::{collections::HashSet, fmt::Debug};

use chrono::{DateTime, Utc};
use database::DatabaseConnector;
use space_traders_client::models::{self, ShipRole};
use utils::{Publisher, Subject};

use crate::{
    cargo::CargoState, error::Result, fuel::FuelState, modules::ModuleState, mounts::MountState,
    my_ship_update::InterShipBroadcaster, status::ShipStatus,
};

use super::ShipManager;

#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct Condition {
    pub condition: f64,
    pub integrity: f64,
}

#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct ConditionState {
    pub engine: Condition,
    pub frame: Condition,
    pub reactor: Condition,
}

#[derive(serde::Serialize)]
pub struct MyShip {
    pub role: database::ShipInfoRole,
    pub status: ShipStatus,
    pub registration_role: ShipRole,
    pub symbol: String,
    pub display_name: String,
    pub engine_speed: i32,
    pub active: bool,
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
    pub is_clone: bool,
    #[serde(skip)]
    pub broadcaster: InterShipBroadcaster,
    #[serde(skip)]
    pub pubsub: Publisher<ShipManager, MyShip>,
}

impl Default for MyShip {
    fn default() -> Self {
        Self {
            active: false,
            is_clone: false,
            cooldown: Default::default(),
            pubsub: Publisher::new(),
            broadcaster: Default::default(),
            role: Default::default(),
            status: Default::default(),
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
        }
    }
}

impl Clone for MyShip {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            role: self.role,
            registration_role: self.registration_role,
            display_name: self.display_name.clone(),
            symbol: self.symbol.clone(),
            active: self.active,
            is_clone: true,
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
            // mpsc: None,
            pubsub: Publisher::new(),
            engine: self.engine,
            reactor: self.reactor,
            frame: self.frame,
        }
    }
}

impl Debug for MyShip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyShip")
            .field("role", &self.role)
            .field("status", &self.status)
            .field("registration_role", &self.registration_role)
            .field("symbol", &self.symbol)
            .field("display_name", &self.display_name)
            .field("engine_speed", &self.engine_speed)
            .field("active", &self.active)
            .field("cooldown_expiration", &self.cooldown_expiration)
            .field("nav", &self.nav)
            .field("cargo", &self.cargo)
            .field("fuel", &self.fuel)
            .field("mounts", &self.mounts)
            .field("conditions", &self.conditions)
            .field("is_clone", &self.is_clone)
            // .field("broadcaster", &self.broadcaster)
            // .field("pubsub", &self.pubsub)
            .finish_non_exhaustive()
    }
}

impl From<&MyShip> for database::ShipState {
    fn from(value: &MyShip) -> Self {
        Self {
            id: 0,
            symbol: value.symbol.clone(),
            display_name: value.display_name.clone(),
            role: value.role,
            active: value.active,
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

impl MyShip {
    pub fn update(&mut self, ship: models::Ship) {
        self.mutate();
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

        // ship.frame.module_slots;
        // ship.frame.mounting_points;
        // ship.frame.requirements.;

        self.conditions.engine.condition = ship.engine.condition;
        self.conditions.engine.integrity = ship.engine.integrity;
        self.conditions.frame.condition = ship.frame.condition;
        self.conditions.frame.integrity = ship.frame.integrity;
        self.conditions.reactor.condition = ship.reactor.condition;
        self.conditions.reactor.integrity = ship.reactor.integrity;
    }

    // pub fn can_mutate(&self) -> bool {
    //     self.is_clone
    // }
    pub fn mutate(&self) {
        if self.is_clone {
            panic!("Cannot mutate a cloned ship");
        }
    }

    pub async fn notify(&self) {
        self.pubsub.notify_observers(self.clone()).await;
    }

    pub async fn apply_from_db(
        &mut self,
        database_pool: database::DbPool,
    ) -> Result<database::ShipInfo> {
        self.mutate();
        let db_ship = database::ShipInfo::get_by_symbol(&database_pool, &self.symbol).await?;
        let ship_info = match db_ship {
            Some(db_ship) => db_ship,
            None => {
                if self.role == database::ShipInfoRole::TempTrader {
                    return Err(crate::error::Error::General(
                        "Ship was a temp trader".to_string(),
                    ));
                }
                let display_name = if self.display_name.is_empty() {
                    self.symbol.clone()
                } else {
                    self.display_name.clone()
                };
                let ship_info = database::ShipInfo {
                    purchase_id: None,
                    symbol: self.symbol.clone(),
                    display_name,
                    role: self.role,
                    active: self.active,
                    assignment_id: None,
                };
                database::ShipInfo::insert(&database_pool, &ship_info).await?;
                ship_info
            }
        };

        self.update_ship_info(ship_info.clone());

        self.notify().await;

        Ok(ship_info)
    }

    fn update_ship_info(&mut self, ship_info: database::ShipInfo) {
        self.mutate();
        self.active = ship_info.active;
        self.display_name = ship_info.display_name;
        self.symbol = ship_info.symbol;
        if self.role != database::ShipInfoRole::TempTrader {
            self.role = ship_info.role;
        }
    }

    pub async fn update_info_db_shipyard(
        ship: models::ShipyardShip,
        database_pool: &database::DbPool,
    ) -> Result<()> {
        database::EngineInfo::insert(database_pool, &database::EngineInfo::from(*ship.engine))
            .await?;
        database::FrameInfo::insert(database_pool, &database::FrameInfo::from(*ship.frame)).await?;
        database::ReactorInfo::insert(database_pool, &database::ReactorInfo::from(*ship.reactor))
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

    pub async fn update_info_db(
        ship: models::Ship,
        database_pool: &database::DbPool,
    ) -> Result<()> {
        database::EngineInfo::insert(database_pool, &database::EngineInfo::from(*ship.engine))
            .await?;
        database::FrameInfo::insert(database_pool, &database::FrameInfo::from(*ship.frame)).await?;
        database::ReactorInfo::insert(database_pool, &database::ReactorInfo::from(*ship.reactor))
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

    pub async fn snapshot(&self, database_pool: &database::DbPool) -> Result<i64> {
        let state = database::ShipState::from(self);

        let id = database::ShipState::insert_get_id(database_pool, &state).await?;

        Ok(id)
    }
}
