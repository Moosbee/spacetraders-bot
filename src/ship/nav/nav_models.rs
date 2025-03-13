use std::{collections::HashMap, fmt::Debug};

use chrono::{DateTime, Utc};
use serde_with::serde_as;
use space_traders_client::models;

use crate::sql;

#[derive(Default, serde::Serialize, Clone)]
pub struct NavigationState {
    pub flight_mode: models::ShipNavFlightMode,
    pub status: models::ShipNavStatus,
    pub system_symbol: String,
    pub waypoint_symbol: String,
    pub route: RouteState,
    pub auto_pilot: Option<AutopilotState>,
    #[serde(skip)]
    pub cache: Cache,
}

impl Debug for NavigationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationState")
            .field("flight_mode", &self.flight_mode)
            .field("status", &self.status)
            .field("system_symbol", &self.system_symbol)
            .field("waypoint_symbol", &self.waypoint_symbol)
            .field("route", &self.route)
            .field("auto_pilot", &self.auto_pilot)
            // .field("cache", &self.cache)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default, serde::Serialize, Clone)]
pub struct RouteState {
    pub arrival: DateTime<Utc>,
    pub departure_time: DateTime<Utc>,
    pub destination_symbol: String,
    pub destination_system_symbol: String,
    pub origin_symbol: String,
    pub origin_system_symbol: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NavMode {
    Burn,
    Cruise,
    Drift,
    BurnAndCruise,
    CruiseAndDrift,
    BurnAndDrift,
    BurnAndCruiseAndDrift,
}

pub(crate) struct Mode {
    pub radius: f64,
    pub cost_multiplier: f64,
    pub mode: models::ShipNavFlightMode,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct RouteInstruction {
    pub start_symbol: String,
    pub end_symbol: String,
    pub flight_mode: models::ShipNavFlightMode,
    pub start_is_marketplace: bool,

    pub distance: f64,

    /// The amount of fuel that needs to be in the Tanks to do the Current jump
    pub refuel_to: i32,

    /// The amount of fuel in the cargo to get to the next Market
    pub fuel_in_cargo: i32,
}

#[serde_as]
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionDetails {
    pub start: sql::Waypoint,
    pub end: sql::Waypoint,
    pub flight_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouteConnection {
    pub start_symbol: String,
    pub end_symbol: String,
    pub flight_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub cost: f64,
    pub re_cost: f64,
}

#[derive(Clone, Default, serde::Serialize)]
pub struct AutopilotState {
    pub arrival: DateTime<Utc>,
    pub departure_time: DateTime<Utc>,
    pub destination_symbol: String,
    pub destination_system_symbol: String,
    pub origin_symbol: String,
    pub origin_system_symbol: String,
    pub distance: f64,
    pub fuel_cost: i32,
    pub instructions: Vec<RouteInstruction>,
    pub connections: Vec<super::nav_models::ConnectionDetails>,
    pub travel_time: f64,
}

impl Debug for AutopilotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutopilotState")
            .field("arrival", &self.arrival)
            .field("departure_time", &self.departure_time)
            .field("destination_symbol", &self.destination_symbol)
            .field("destination_system_symbol", &self.destination_system_symbol)
            .field("origin_symbol", &self.origin_symbol)
            .field("origin_system_symbol", &self.origin_system_symbol)
            .field("distance", &self.distance)
            .field("fuel_cost", &self.fuel_cost)
            // .field("instructions", &self.instructions)
            // .field("connections", &self.connections)
            .field("travel_time", &self.travel_time)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Cache {
    pub routes: HashMap<(String, String, NavMode, bool, i32, i32), Vec<RouteConnection>>,
}
