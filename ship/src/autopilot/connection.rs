use async_graphql::{Enum, SimpleObject, Union};
use space_traders_client::models::{self, ShipNavFlightMode};
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
    pub cooldown_time: f64,
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
    pub fuel_needed: i32,
    pub fuel_required: i32,
    pub start_is_marketplace: bool,
}

#[derive(Clone, Default, serde::Serialize, Debug)]
pub struct Route {
    pub connections: Vec<ConcreteConnection>,
    pub total_distance: f64,
    pub total_fuel_cost: f64,
    pub total_travel_time: f64,
    pub total_api_requests: i32,
}

pub enum ConnectionTypeGQL {
    JumpGate,
    Warp,
    Navigate,
}

impl From<ConnectionType> for ConnectionTypeGQL {
    fn from(connection_type: ConnectionType) -> Self {
        match connection_type {
            ConnectionType::JumpGate => ConnectionTypeGQL::JumpGate,
            ConnectionType::Warp { .. } => ConnectionTypeGQL::Warp,
            ConnectionType::Navigate { .. } => ConnectionTypeGQL::Navigate,
        }
    }
}

// Connection variants as GraphQL objects
#[derive(Debug, Clone, serde::Serialize, SimpleObject)]
pub struct JumpConnectionGQL {
    pub start_symbol: String,
    pub end_symbol: String,
    pub distance: f64,
    pub cooldown_time: f64,
}

#[derive(Debug, Clone, serde::Serialize, SimpleObject)]
pub struct WarpConnectionGQL {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: RefuelGQL,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[derive(Debug, Clone, serde::Serialize, SimpleObject)]
pub struct NavigateConnectionGQL {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: RefuelGQL,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[derive(Debug, Clone, serde::Serialize, Union)]
pub enum ConcreteConnectionGQL {
    JumpGate(JumpConnectionGQL),
    Warp(WarpConnectionGQL),
    Navigate(NavigateConnectionGQL),
}

#[derive(Debug, Clone, serde::Serialize, SimpleObject)]
pub struct RefuelGQL {
    /// The amount of fuel in fuel units that needs to be in the Tanks to do the Current jump
    pub fuel_needed: i32,
    /// the amount of fuel trade units to be in the cargo hold, needed to reach the next Marketplace
    pub fuel_required: i32,
    pub start_is_marketplace: bool,
}

#[derive(Clone, Default, serde::Serialize, SimpleObject)]
pub struct RouteGQL {
    pub connections: Vec<ConcreteConnectionGQL>,
    pub total_distance: f64,
    pub total_fuel_cost: f64,
    pub total_travel_time: f64,
    pub total_api_requests: i32,
}

// Conversions
impl From<Refuel> for RefuelGQL {
    fn from(refuel: Refuel) -> Self {
        RefuelGQL {
            fuel_needed: refuel.fuel_needed,
            fuel_required: refuel.fuel_required,
            start_is_marketplace: refuel.start_is_marketplace,
        }
    }
}

impl From<JumpConnection> for JumpConnectionGQL {
    fn from(conn: JumpConnection) -> Self {
        JumpConnectionGQL {
            start_symbol: conn.start_symbol,
            end_symbol: conn.end_symbol,
            distance: conn.distance,
            cooldown_time: conn.cooldown_time,
        }
    }
}

impl From<WarpConnection> for WarpConnectionGQL {
    fn from(conn: WarpConnection) -> Self {
        WarpConnectionGQL {
            start_symbol: conn.start_symbol,
            end_symbol: conn.end_symbol,
            nav_mode: conn.nav_mode,
            distance: conn.distance,
            travel_time: conn.travel_time,
            refuel: conn.refuel.into(),
            start_is_marketplace: conn.start_is_marketplace,
            end_is_marketplace: conn.end_is_marketplace,
        }
    }
}

impl From<NavigateConnection> for NavigateConnectionGQL {
    fn from(conn: NavigateConnection) -> Self {
        NavigateConnectionGQL {
            start_symbol: conn.start_symbol,
            end_symbol: conn.end_symbol,
            nav_mode: conn.nav_mode,
            distance: conn.distance,
            travel_time: conn.travel_time,
            refuel: conn.refuel.into(),
            start_is_marketplace: conn.start_is_marketplace,
            end_is_marketplace: conn.end_is_marketplace,
        }
    }
}

impl From<ConcreteConnection> for ConcreteConnectionGQL {
    fn from(conn: ConcreteConnection) -> Self {
        match conn {
            ConcreteConnection::JumpGate(jump) => ConcreteConnectionGQL::JumpGate(jump.into()),
            ConcreteConnection::Warp(warp) => ConcreteConnectionGQL::Warp(warp.into()),
            ConcreteConnection::Navigate(nav) => ConcreteConnectionGQL::Navigate(nav.into()),
        }
    }
}

impl From<Route> for RouteGQL {
    fn from(route: Route) -> Self {
        RouteGQL {
            connections: route.connections.into_iter().map(|c| c.into()).collect(),
            total_distance: route.total_distance,
            total_fuel_cost: route.total_fuel_cost,
            total_travel_time: route.total_travel_time,
            total_api_requests: route.total_api_requests,
        }
    }
}
