use space_traders_client::models;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct SimpleConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub connection_type: ConnectionType,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
    pub cost: f64,
    pub re_cost: f64,
    pub distance: f64,
}

impl PartialEq for SimpleConnection {
    fn eq(&self, other: &Self) -> bool {
        self.start_symbol == other.start_symbol
            && self.end_symbol == other.end_symbol
            && self.connection_type == other.connection_type
    }
}

impl Hash for SimpleConnection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start_symbol.hash(state);
        self.end_symbol.hash(state);
        self.connection_type.hash(state);
    }
}

impl Eq for SimpleConnection {}

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum ConnectionType {
    JumpGate,
    Warp { nav_mode: models::ShipNavFlightMode },
    Navigate { nav_mode: models::ShipNavFlightMode },
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ConcreteConnection {
    JumpGate(JumpConnection),
    Warp(WarpConnection),
    Navigate(NavigateConnection),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct JumpConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub distance: f64,
    // pub cooldown_time: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WarpConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: Refuel,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NavigateConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: Refuel,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Refuel {
    /// The amount of fuel in fuel units that needs to be in the Tanks to do the Current jump
    pub fuel_needed: i32,
    /// the amount of fuel trade units to be in the cargo hold, needed to reach the next Marketplace
    pub fuel_required: i32,
    pub start_is_marketplace: bool,
}

#[derive(Clone, Default, serde::Serialize, Debug)]
pub struct Route {
    pub connections: Vec<ConcreteConnection>,
    pub total_distance: f64,
    pub total_fuel_cost: f64,
    pub total_travel_time: f64,
}
