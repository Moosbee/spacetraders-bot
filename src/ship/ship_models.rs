use chrono::{DateTime, Utc};
use space_traders_client::models::{self, ShipRole, TradeSymbol};

use crate::{types::Subject, workers::mining::m_types::MiningShipAssignment};

use super::ShipManager;

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
    pub pubsub: crate::types::Publisher<ShipManager, MyShip>,
}

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
            // mpsc: None,
            pubsub: crate::types::Publisher::new(),
        }
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
    pub inventory: Vec<(TradeSymbol, i32)>,
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
    pub fn from_ship(ship: models::Ship) -> MyShip {
        let mut new_ship = MyShip::default();
        new_ship.update(ship);
        new_ship
    }

    pub fn update(&mut self, ship: models::Ship) {
        self.symbol = ship.symbol;
        self.engine_speed = ship.engine.speed;
        self.registration_role = ship.registration.role;
        self.cooldown_expiration =
            DateTime::parse_from_rfc3339(&ship.cooldown.expiration.unwrap_or("".to_string()))
                .map(|op| op.to_utc())
                .ok();
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

    // pub fn is_on_cooldown(&self) -> bool {
    //     if self.cooldown_expiration.is_some() {
    //         let t = self.cooldown_expiration.unwrap();
    //         let t = t - Utc::now();
    //         let t = t.num_seconds();
    //         t > 0
    //     } else {
    //         true
    //     }
    // }

    // pub async fn wait_for_cooldown(&self) -> anyhow::Result<()> {
    //     if self.cooldown_expiration.is_none() {
    //         return Err(anyhow::anyhow!("Is not on cooldown"));
    //     }
    //     let t = self.cooldown_expiration.unwrap();
    //     let t = t - Utc::now();
    //     let t = t.num_seconds().try_into()?;
    //     tokio::time::sleep(std::time::Duration::from_secs(t)).await;
    //     Ok(())
    // }
}
