use anyhow::Ok;
use chrono::{DateTime, Utc};
use space_traders_client::models;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    Construction,
    Trader,
    Contract,
    Scraper,
    Mining,
    Manuel,
}

#[derive(Debug)]
pub struct MyShip {
    pub role: Role,
    registration_role: models::ShipRole,
    symbol: String,
    engine_speed: i32,
    cooldown_expiration: Option<DateTime<Utc>>,
    cargo_capacity: i32,
    cargo_units: i32,
    cargo: Vec<(models::TradeSymbol, i32)>,
    fuel_capacity: i32,
    fuel_current: i32,
    nav_flight_mode: models::ShipNavFlightMode,
    nav_status: models::ShipNavStatus,
    nav_system_symbol: String,
    nav_waypoint_symbol: String,
    nav_route_arrival: DateTime<Utc>,
    nav_route_departure_time: DateTime<Utc>,
    nav_route_destination_symbol: String,
    nav_route_destination_system_symbol: String,
    nav_route_origin_symbol: String,
    nav_route_origin_system_symbol: String,
}

impl MyShip {
    pub fn new(ship: models::Ship) -> MyShip {
        let cargo: Vec<(models::TradeSymbol, i32)> = ship
            .cargo
            .inventory
            .iter()
            .map(|f| (f.symbol, f.units))
            .collect();
        // ship.engine.condition
        // ship.engine.integrity
        MyShip {
            symbol: ship.symbol,
            role: Role::Manuel,
            engine_speed: ship.engine.speed,
            registration_role: ship.registration.role,
            cooldown_expiration: DateTime::parse_from_rfc3339(
                &ship.cooldown.expiration.unwrap_or("".to_string()),
            )
            .map(|op| op.to_utc())
            .ok(),
            cargo_capacity: ship.cargo.capacity,
            cargo_units: ship.cargo.units,
            cargo,
            fuel_capacity: ship.fuel.capacity,
            fuel_current: ship.fuel.current,
            nav_flight_mode: ship.nav.flight_mode,
            nav_status: ship.nav.status,
            nav_system_symbol: ship.nav.system_symbol,
            nav_waypoint_symbol: ship.nav.waypoint_symbol,
            nav_route_arrival: DateTime::parse_from_rfc3339(&ship.nav.route.arrival)
                .unwrap()
                .to_utc(),
            nav_route_departure_time: DateTime::parse_from_rfc3339(&ship.nav.route.departure_time)
                .unwrap()
                .to_utc(),
            nav_route_destination_symbol: ship.nav.route.destination.symbol,
            nav_route_destination_system_symbol: ship.nav.route.destination.system_symbol,
            nav_route_origin_symbol: ship.nav.route.origin.symbol,
            nav_route_origin_system_symbol: ship.nav.route.origin.system_symbol,
        }
    }
    pub fn from_ship(ship: models::Ship) -> MyShip {
        MyShip::new(ship)
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

    pub fn is_in_transit(&self) -> bool {
        if self.nav_status == models::ShipNavStatus::InTransit {
            let t = self.nav_route_arrival - Utc::now();
            let t = t.num_seconds();
            t > 0
        } else {
            false
        }
    }

    pub async fn wait_for_transit(&self) -> anyhow::Result<()> {
        let t = self.nav_route_arrival - Utc::now();
        let t = t.num_seconds().try_into()?;
        tokio::time::sleep(std::time::Duration::from_secs(t)).await;
        Ok(())
    }
}
