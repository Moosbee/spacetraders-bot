use space_traders_client::models;

#[derive(Debug)]
pub struct MyShip {
    engine_speed: i32,
    registration_role: models::ShipRole,
    cooldown_expiration: Option<String>,
    cargo_capacity: i32,
    cargo_units: i32,
    cargo: Vec<(models::TradeSymbol, i32)>,
    fuel_capacity: i32,
    fuel_current: i32,
    nav_flight_mode: models::ShipNavFlightMode,
    nav_status: models::ShipNavStatus,
    nav_system_symbol: String,
    nav_waypoint_symbol: String,
    nav_route_arrival: String,
    nav_route_departure_time: String,
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
            engine_speed: ship.engine.speed,
            registration_role: ship.registration.role,
            cooldown_expiration: ship.cooldown.expiration,
            cargo_capacity: ship.cargo.capacity,
            cargo_units: ship.cargo.units,
            cargo,
            fuel_capacity: ship.fuel.capacity,
            fuel_current: ship.fuel.current,
            nav_flight_mode: ship.nav.flight_mode,
            nav_status: ship.nav.status,
            nav_system_symbol: ship.nav.system_symbol,
            nav_waypoint_symbol: ship.nav.waypoint_symbol,
            nav_route_arrival: ship.nav.route.arrival,
            nav_route_departure_time: ship.nav.route.departure_time,
            nav_route_destination_symbol: ship.nav.route.destination.symbol,
            nav_route_destination_system_symbol: ship.nav.route.destination.system_symbol,
            nav_route_origin_symbol: ship.nav.route.origin.symbol,
            nav_route_origin_system_symbol: ship.nav.route.origin.system_symbol,
        }
    }
    pub fn from_ship(ship: models::Ship) -> MyShip {
        MyShip::new(ship)
    }
}
