use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use chrono::{DateTime, Utc};
use futures::FutureExt;
use log::debug;
use my_ship_update::{InterShipBroadcaster, MyShipUpdate, ShipUpdate};
use space_traders_client::models::{self, ShipRole, TradeSymbol};
use tokio::select;

use crate::{
    api,
    pilot::MiningShipAssignment,
    sql::{self, DatabaseConnector},
    types::{SendFuture, Subject},
};

use crate::error::Result;

use super::ShipManager;

impl Default for MyShip {
    fn default() -> Self {
        Self {
            active: false,
            is_clone: false,
            pubsub: crate::types::Publisher::new(),
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
            role: self.role.clone(),
            registration_role: self.registration_role,
            display_name: self.display_name.clone(),
            symbol: self.symbol.clone(),
            active: self.active,
            is_clone: true,
            engine_speed: self.engine_speed,
            cooldown_expiration: self.cooldown_expiration,
            modules: self.modules.clone(),
            nav: self.nav.clone(),
            cargo: self.cargo.clone(),
            fuel: self.fuel.clone(),
            mounts: self.mounts.clone(),
            conditions: self.conditions.clone(),
            broadcaster: self.broadcaster.clone(),
            // mpsc: None,
            pubsub: crate::types::Publisher::new(),
            engine: self.engine,
            reactor: self.reactor,
            frame: self.frame,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub enum ShippingStatus {
    InTransitToPurchase,
    Purchasing,
    InTransitToDelivery,
    Delivering,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ShipStatus {
    Construction {
        cycle: Option<i32>,
        shipment_id: Option<i64>,
        shipping_status: Option<ShippingStatus>,
        waiting_for_manager: bool,
    },
    Trader {
        shipment_id: Option<i32>,
        cycle: Option<i32>,
        shipping_status: Option<ShippingStatus>,
        waiting_for_manager: bool,
    },
    Contract {
        contract_id: Option<String>,
        run_id: Option<i32>,
        cycle: Option<i32>,
        shipping_status: Option<ShippingStatus>,
        waiting_for_manager: bool,
    },
    Scraper {
        cycle: Option<i32>,
        waiting_for_manager: bool,
        waypoint_symbol: Option<String>,
        scrap_date: Option<chrono::DateTime<Utc>>,
    },
    Mining {
        assignment: MiningShipAssignment,
    },
    Charting {
        cycle: Option<i32>,
        waiting_for_manager: bool,
        waypoint_symbol: Option<String>,
    },
    #[default]
    Manuel,
}

#[derive(serde::Serialize)]
pub struct MyShip {
    pub role: crate::sql::ShipInfoRole,
    pub status: ShipStatus,
    pub registration_role: ShipRole,
    pub symbol: String,
    pub display_name: String,
    pub engine_speed: i32,
    pub active: bool,
    pub cooldown_expiration: Option<DateTime<Utc>>,
    // Navigation state
    pub nav: super::nav::nav_models::NavigationState,
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
    pub pubsub: crate::types::Publisher<ShipManager, MyShip>,
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
pub mod my_ship_update {
    use space_traders_client::models::TradeSymbol;
    use tokio::sync::broadcast;

    #[derive(Debug)]
    pub struct InterShipBroadcaster {
        pub sender: tokio::sync::broadcast::Sender<MyShipUpdate>,
        pub receiver: tokio::sync::broadcast::Receiver<MyShipUpdate>,
    }

    impl Default for InterShipBroadcaster {
        fn default() -> Self {
            let (sender, receiver) = broadcast::channel(16);
            Self { sender, receiver }
        }
    }

    impl Clone for InterShipBroadcaster {
        fn clone(&self) -> Self {
            Self {
                sender: self.sender.clone(),
                receiver: self.receiver.resubscribe(),
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct MyShipUpdate {
        pub symbol: String,
        pub update: ShipUpdate,
    }

    #[derive(Debug, Clone, Default)]
    pub enum ShipUpdate {
        CargoChange(CargoChange),
        TransferRequest(TransferRequest),
        #[default]
        None,
    }

    #[derive(Debug, Clone, Default)]
    pub struct CargoChange {
        pub units: i32,
        pub trade_symbol: TradeSymbol,
    }

    #[derive(Debug, Clone)]
    pub struct TransferRequest {
        pub units: i32,
        pub trade_symbol: TradeSymbol,
        pub target: String,
        pub callback: tokio::sync::mpsc::Sender<()>,
    }
}
#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct MountState {
    pub mounts: Vec<models::ship_mount::Symbol>,
}

#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct ModuleState {
    pub modules: Vec<models::ship_module::Symbol>,
}

#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct CargoState {
    pub capacity: i32,
    pub units: i32,
    pub inventory: HashMap<TradeSymbol, i32>,
}

#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct FuelState {
    pub capacity: i32,
    pub current: i32,
}

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

impl MyShip {
    pub fn from_ship(ship: models::Ship, broadcaster: InterShipBroadcaster) -> MyShip {
        let mut new_ship = MyShip::default();
        new_ship.update(ship);
        new_ship.broadcaster = broadcaster;
        new_ship
    }

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
    pub async fn try_recive_update(&mut self, api: &api::Api) {
        self.mutate();
        while let Ok(data) = self.broadcaster.receiver.try_recv() {
            self.handle_update(data, api).await;
        }
    }

    async fn receive_update_loop(
        &mut self,
        cancel: &tokio_util::sync::CancellationToken,
        api: &api::Api,
    ) {
        loop {
            let data = select! {
                data = self.broadcaster.receiver.recv() => data,
                _ = cancel.cancelled() => break,
            };
            if let Ok(data) = data {
                self.handle_update(data, api).await;
            }
            if cancel.is_cancelled() {
                break;
            }
        }
    }

    async fn handle_update(&mut self, data: MyShipUpdate, api: &api::Api) {
        if data.symbol != self.symbol {
            return;
        }
        debug!(
            "Handling update: {:?} for ship: {}",
            data.update, self.symbol
        );
        let erg: std::result::Result<(), crate::error::Error> = match data.update {
            ShipUpdate::CargoChange(cargo_change) => self
                .cargo
                .handle_cago_update(cargo_change.units, cargo_change.trade_symbol),
            ShipUpdate::TransferRequest(transfer_request) => {
                let erg = self
                    .transfer_cargo(
                        transfer_request.trade_symbol,
                        transfer_request.units,
                        api,
                        &transfer_request.target,
                    )
                    .await;
                debug!("Transfer cargo: {:?} {}", erg, self.symbol);
                let _reg: std::result::Result<(), tokio::sync::mpsc::error::SendError<()>> =
                    transfer_request.callback.send(()).await;
                erg.map(|_| ())
            }
            ShipUpdate::None => Ok(()),
        };
        if let Err(e) = erg {
            log::error!("Failed to handle update: {}", e);
        }
        self.notify().await;
    }

    #[deprecated]
    pub async fn sleep(&mut self, duration: std::time::Duration, api: &api::Api) {
        self.mutate();
        let cancel = tokio_util::sync::CancellationToken::new();

        let update_future = self.receive_update_loop(&cancel, api);
        let sleep_future = tokio::time::sleep(duration).then(|_| {
            cancel.cancel();

            async move {}
        });
        let _erg = futures::future::join(update_future, sleep_future)
            .send() // https://github.com/rust-lang/rust/issues/96865
            .await;
    }

    pub async fn apply_from_db(
        &mut self,
        database_pool: crate::sql::DbPool,
    ) -> Result<crate::sql::ShipInfo> {
        self.mutate();
        let db_ship = crate::sql::ShipInfo::get_by_symbol(&database_pool, &self.symbol).await?;
        let ship_info = match db_ship {
            Some(db_ship) => db_ship,
            None => {
                if self.role == sql::ShipInfoRole::TempTrader {
                    return Err(crate::error::Error::General(
                        "Ship was a temp trader".to_string(),
                    ));
                }
                let display_name = if self.display_name.is_empty() {
                    self.symbol.clone()
                } else {
                    self.display_name.clone()
                };
                let ship_info = crate::sql::ShipInfo {
                    symbol: self.symbol.clone(),
                    display_name,
                    role: self.role.clone(),
                    active: self.active,
                };
                crate::sql::ShipInfo::insert(&database_pool, &ship_info).await?;
                ship_info
            }
        };

        self.update_ship_info(ship_info.clone());

        self.notify().await;

        Ok(ship_info)
    }

    fn update_ship_info(&mut self, ship_info: crate::sql::ShipInfo) {
        self.mutate();
        self.active = ship_info.active;
        self.display_name = ship_info.display_name;
        self.symbol = ship_info.symbol;
        if self.role != sql::ShipInfoRole::TempTrader {
            self.role = ship_info.role;
        }
    }

    pub async fn update_info_db_shipyard(
        ship: models::ShipyardShip,
        database_pool: &crate::sql::DbPool,
    ) -> Result<()> {
        sql::EngineInfo::insert(database_pool, &sql::EngineInfo::from(*ship.engine)).await?;
        sql::FrameInfo::insert(database_pool, &sql::FrameInfo::from(*ship.frame)).await?;
        sql::ReactorInfo::insert(database_pool, &sql::ReactorInfo::from(*ship.reactor)).await?;

        sql::ModuleInfo::insert_bulk(
            database_pool,
            &ship
                .modules
                .into_iter()
                .map(sql::ModuleInfo::from)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .await?;
        sql::MountInfo::insert_bulk(
            database_pool,
            &ship
                .mounts
                .into_iter()
                .map(sql::MountInfo::from)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .await?;
        Ok(())
    }

    pub async fn update_info_db(
        ship: models::Ship,
        database_pool: &crate::sql::DbPool,
    ) -> Result<()> {
        sql::EngineInfo::insert(database_pool, &sql::EngineInfo::from(*ship.engine)).await?;
        sql::FrameInfo::insert(database_pool, &sql::FrameInfo::from(*ship.frame)).await?;
        sql::ReactorInfo::insert(database_pool, &sql::ReactorInfo::from(*ship.reactor)).await?;

        sql::ModuleInfo::insert_bulk(
            database_pool,
            &ship
                .modules
                .into_iter()
                .map(sql::ModuleInfo::from)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .await?;
        sql::MountInfo::insert_bulk(
            database_pool,
            &ship
                .mounts
                .into_iter()
                .map(sql::MountInfo::from)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .await?;
        Ok(())
    }

    pub async fn snapshot(&self, database_pool: &crate::sql::DbPool) -> Result<i64> {
        let state = sql::ShipState::from(self);

        let id = sql::ShipState::insert_get_id(database_pool, &state).await?;

        Ok(id)
    }
}
