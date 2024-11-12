use chrono::{DateTime, Utc};
use space_traders_client::models::{self, ShipNavFlightMode, ShipNavStatus, ShipRole, TradeSymbol};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Role {
    Construction,
    Trader,
    Contract,
    Scraper,
    Mining,
    #[default]
    Manuel,
}

#[derive(Debug, Default)]
pub struct MyShip {
    pub role: Role,
    pub registration_role: ShipRole,
    pub symbol: String,
    pub engine_speed: i32,
    pub cooldown_expiration: Option<DateTime<Utc>>,
    // Navigation state
    pub nav: NavigationState,
    // Cargo state
    pub cargo: CargoState,
    // Fuel state
    pub fuel: FuelState,
}

#[derive(Debug, Default)]
pub struct NavigationState {
    pub flight_mode: ShipNavFlightMode,
    pub status: ShipNavStatus,
    pub system_symbol: String,
    pub waypoint_symbol: String,
    pub route: RouteState,
}

#[derive(Debug, Default)]
pub struct RouteState {
    pub arrival: DateTime<Utc>,
    pub departure_time: DateTime<Utc>,
    pub destination_symbol: String,
    pub destination_system_symbol: String,
    pub origin_symbol: String,
    pub origin_system_symbol: String,
}

#[derive(Debug, Default)]
pub struct CargoState {
    pub capacity: i32,
    pub units: i32,
    pub inventory: Vec<(TradeSymbol, i32)>,
}

#[derive(Debug, Default)]
pub struct FuelState {
    pub capacity: i32,
    pub current: i32,
}

#[derive(Debug)]
pub struct RouteInstruction {
    pub start_symbol: String,
    pub end_symbol: String,
    pub flight_mode: models::ShipNavFlightMode,
    pub start_is_marketplace: bool,

    /// The amount of fuel that needs to be in the Tanks to do the Current jump
    pub refuel_to: i32,

    /// The amount of fuel in the cargo to get to the next Market
    pub fuel_in_cargo: i32,
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
    }

    pub fn is_on_cooldown(&self) -> bool {
        if self.cooldown_expiration.is_some() {
            let t = self.cooldown_expiration.unwrap();
            let t = t - Utc::now();
            let t = t.num_seconds();
            t > 0
        } else {
            true
        }
    }

    pub async fn wait_for_cooldown(&self) -> anyhow::Result<()> {
        if self.cooldown_expiration.is_none() {
            return Err(anyhow::anyhow!("Is not on cooldown"));
        }
        let t = self.cooldown_expiration.unwrap();
        let t = t - Utc::now();
        let t = t.num_seconds().try_into()?;
        tokio::time::sleep(std::time::Duration::from_secs(t)).await;
        Ok(())
    }
}
