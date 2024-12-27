use std::collections::HashMap;

use chrono::{DateTime, Utc};
use futures::FutureExt;
use log::debug;
use my_ship_update::{InterShipBroadcaster, MyShipUpdate, ShipUpdate};
use space_traders_client::models::{self, ShipRole, TradeSymbol};
use tokio::select;

use crate::{
    api,
    types::{SendFuture, Subject},
    workers::mining::m_types::MiningShipAssignment,
};

use super::ShipManager;

impl Default for MyShip {
    fn default() -> Self {
        let me = Self {
            role: Default::default(),
            registration_role: Default::default(),
            symbol: Default::default(),
            engine_speed: Default::default(),
            cooldown_expiration: Default::default(),
            nav: Default::default(),
            cargo: Default::default(),
            fuel: Default::default(),
            mounts: Default::default(),
            conditions: Default::default(),
            broadcaster: Default::default(),
            pubsub: crate::types::Publisher::new(),
        };
        me
    }
}

impl Clone for MyShip {
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }

    fn clone(&self) -> Self {
        Self {
            role: self.role.clone(),
            registration_role: self.registration_role.clone(),
            symbol: self.symbol.clone(),
            engine_speed: self.engine_speed.clone(),
            cooldown_expiration: self.cooldown_expiration.clone(),
            nav: self.nav.clone(),
            cargo: self.cargo.clone(),
            fuel: self.fuel.clone(),
            mounts: self.mounts.clone(),
            conditions: self.conditions.clone(),
            broadcaster: self.broadcaster.clone(),
            // mpsc: None,
            pubsub: crate::types::Publisher::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Role {
    Construction,
    Trader(Option<(i32, u32)>),
    Contract(Option<(String, i32)>),
    Scraper,
    Mining(MiningShipAssignment),
    #[default]
    Manuel,
}

#[derive(Debug, serde::Serialize)]
pub struct MyShip {
    pub role: Role,
    pub registration_role: ShipRole,
    pub symbol: String,
    pub engine_speed: i32,
    pub cooldown_expiration: Option<DateTime<Utc>>,
    // Navigation state
    pub nav: super::nav::nav_models::NavigationState,
    // Cargo state
    pub cargo: CargoState,
    // Fuel state
    pub fuel: FuelState,
    // Mining state
    pub mounts: MountState,
    // Conditions
    pub conditions: ConditionState,
    #[serde(skip)]
    pub broadcaster: InterShipBroadcaster,
    #[serde(skip)]
    pub pubsub: crate::types::Publisher<ShipManager, MyShip>,
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
        self.symbol = ship.symbol;
        self.engine_speed = ship.engine.speed;
        self.registration_role = ship.registration.role;
        self.update_cooldown(&ship.cooldown);
        self.nav.update(&ship.nav);
        self.cargo.update(&ship.cargo);
        self.fuel.update(&ship.fuel);
        self.mounts.update(&ship.mounts);

        self.conditions.engine.condition = ship.engine.condition;
        self.conditions.engine.integrity = ship.engine.integrity;
        self.conditions.frame.condition = ship.frame.condition;
        self.conditions.frame.integrity = ship.frame.integrity;
        self.conditions.reactor.condition = ship.reactor.condition;
        self.conditions.reactor.integrity = ship.reactor.integrity;
    }

    pub async fn notify(&self) {
        self.pubsub.notify_observers(self.clone()).await;
    }
    pub async fn try_recive_update(&mut self, api: &api::Api) {
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
        ()
    }

    async fn handle_update(&mut self, data: MyShipUpdate, api: &api::Api) {
        if data.symbol != self.symbol {
            return;
        }
        debug!(
            "Handling update: {:?} for ship: {}",
            data.update, self.symbol
        );
        let erg = match data.update {
            ShipUpdate::CargoChange(cargo_change) => self.cargo.handle_cago_update(cargo_change),
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
                let _reg = transfer_request.callback.send(()).await;
                erg.map(|_| ())
            }
            ShipUpdate::None => Ok(()),
        };
        if let Err(e) = erg {
            log::error!(
                "Failed to handle update: {} {:?} {:?} {:?}",
                e,
                e.root_cause(),
                e.source(),
                e.backtrace()
            );
        }
        self.notify().await;
    }

    pub async fn sleep(&mut self, duration: std::time::Duration, api: &api::Api) {
        let cancel = tokio_util::sync::CancellationToken::new();

        let update_future = self.receive_update_loop(&cancel, api);
        let sleep_future = tokio::time::sleep(duration).then(|_| {
            cancel.cancel();

            async move { () }
        });
        let _erg = futures::future::join(update_future, sleep_future)
            .send() // https://github.com/rust-lang/rust/issues/96865
            .await;
    }
}
