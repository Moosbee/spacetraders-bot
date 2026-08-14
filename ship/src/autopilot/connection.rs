use async_graphql::SimpleObject;
use space_traders_client::models;
use std::hash::Hash;

use crate::autopilot::utils::{calculate_jump_cooldown, get_travel_stats};

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

#[derive(Debug, Clone, serde::Serialize, SimpleObject)]
pub struct Refuel {
    /// fuel needed to have in fuel tank to make the jump
    pub fuel_needed: i32,
    /// how many items of fuel need to be in storage at this point to get to the next waypoint
    pub fuel_required: i32,
    pub start_is_marketplace: bool,
}

#[derive(Clone, Default, serde::Serialize, Debug)]
pub struct Route {
    pub connections: Vec<ConcreteConnection>,
    pub total_distance: f64,
    /// time in seconds spent warping or navigating
    pub travel_time: f64,
    // time in seconds spent having a jump cooldown
    pub total_jump_cooldown_time: f64,
    // time in seconds spent in traveling from start to end
    pub total_travel_time: f64,
    pub total_api_requests: i32,
    /// how many fuel tank units are needed
    pub total_fuel: i32,
    /// how many items of fuel are needed, 1 item of fuel = 100 fuel units in fuel tank
    pub total_refuel: i32,
    /// how much all the items of fuel cost
    pub total_fuel_cost: i32,
    /// how many items of anti-matter are needed
    pub total_anti_matter: i32,
    /// how much all the items of anti-matter cost
    pub total_anti_matter_cost: i32,
}

pub fn to_connection(
    connections: &[SimpleConnection],
    ship_stats: &super::ShipNavStats,
) -> Vec<ConcreteConnection> {
    let mut real_route = vec![];

    let mut needed_fuel = 0; // items of fuel in the cargo hold

    for c in connections.iter().rev() {
        match c.connection_type {
            ConnectionType::JumpGate => {
                real_route.push(ConcreteConnection::JumpGate(JumpConnection {
                    start_symbol: c.start_symbol.clone(),
                    end_symbol: c.end_symbol.clone(),
                    distance: c.distance,
                    cooldown_time: calculate_jump_cooldown(c.distance),
                }))
            }
            ConnectionType::Warp { nav_mode } => {
                let stats = get_travel_stats(
                    ship_stats.engine_speed,
                    nav_mode,
                    ship_stats.engine_condition,
                    c.distance,
                );
                if c.start_is_marketplace {
                    needed_fuel = 0;
                } else {
                    needed_fuel += ((stats.fuel_cost as f64) / 100.0).ceil() as i32;
                }
                let refuel = Refuel {
                    fuel_needed: stats.fuel_cost,
                    fuel_required: needed_fuel,
                    start_is_marketplace: c.start_is_marketplace,
                };
                real_route.push(ConcreteConnection::Warp(WarpConnection {
                    start_symbol: c.start_symbol.clone(),
                    end_symbol: c.end_symbol.clone(),
                    nav_mode,
                    distance: stats.distance,
                    travel_time: stats.travel_time,
                    refuel,
                    start_is_marketplace: c.start_is_marketplace,
                    end_is_marketplace: c.end_is_marketplace,
                }))
            }
            ConnectionType::Navigate { nav_mode } => {
                let stats = get_travel_stats(
                    ship_stats.engine_speed,
                    nav_mode,
                    ship_stats.engine_condition,
                    c.distance,
                );
                let refuel = Refuel {
                    fuel_needed: stats.fuel_cost,
                    fuel_required: needed_fuel,
                    start_is_marketplace: c.start_is_marketplace,
                };
                if c.start_is_marketplace {
                    needed_fuel = 0;
                } else {
                    needed_fuel += ((stats.fuel_cost as f64) / 100.0).ceil() as i32;
                }
                real_route.push(ConcreteConnection::Navigate(NavigateConnection {
                    start_symbol: c.start_symbol.clone(),
                    end_symbol: c.end_symbol.clone(),
                    nav_mode,
                    distance: stats.distance,
                    travel_time: stats.travel_time,
                    refuel,
                    start_is_marketplace: c.start_is_marketplace,
                    end_is_marketplace: c.end_is_marketplace,
                }))
            }
        }
    }

    real_route.reverse();
    real_route
}

pub async fn assemble_route(
    connections: &[SimpleConnection],
    ship_stats: &super::ShipNavStats,
    price_getter: &mut impl super::travel_price::TravelPriceCalc,
) -> Result<Route, crate::error::Error> {
    let simple = to_connection(connections, ship_stats);

    let mut distance = 0.0;
    let mut travel_time = 0.0;
    let mut jump_cooldown = 0.0;
    let mut last_jump_cooldown = 0.0;
    let mut api_requests = 0;

    let mut total_fuel_items_needed = 0;
    let mut total_fuel_needed = 0;
    let mut antimatter_needed = 0;

    let mut fuel_cost = 0;
    let mut antimatter_cost = 0;

    for c in simple.iter() {
        match c {
            ConcreteConnection::JumpGate(jump_connection) => {
                distance += jump_connection.distance;
                antimatter_needed += 1;
                antimatter_cost += price_getter
                    .get_antimatter_price(&jump_connection.start_symbol)
                    .await?;
                api_requests += 1;
                jump_cooldown += jump_connection.cooldown_time;
                last_jump_cooldown = jump_cooldown;
            }
            ConcreteConnection::Warp(warp_connection) => {
                distance += warp_connection.distance;
                travel_time += warp_connection.travel_time + 2.0;
                api_requests += 4;

                total_fuel_needed += warp_connection.refuel.fuel_needed;
                if warp_connection.refuel.start_is_marketplace {
                    let fuel_needed_for_current_jump =
                        ((warp_connection.refuel.fuel_needed as f64) / 100.0).ceil() as i32;
                    let fuel_needed_to_purchase_here =
                        warp_connection.refuel.fuel_required + fuel_needed_for_current_jump;
                    total_fuel_items_needed += fuel_needed_to_purchase_here;
                    fuel_cost += price_getter
                        .get_fuel_price(&warp_connection.start_symbol)
                        .await?
                        * fuel_needed_to_purchase_here;
                }
            }
            ConcreteConnection::Navigate(navigate_connection) => {
                distance += navigate_connection.distance;
                travel_time += navigate_connection.travel_time + 2.0;
                api_requests += 4;

                total_fuel_needed += navigate_connection.refuel.fuel_needed;
                if navigate_connection.refuel.start_is_marketplace {
                    let fuel_needed_for_current_jump =
                        ((navigate_connection.refuel.fuel_needed as f64) / 100.0).ceil() as i32;
                    let fuel_needed_to_purchase_here =
                        navigate_connection.refuel.fuel_required + fuel_needed_for_current_jump;
                    total_fuel_items_needed += fuel_needed_to_purchase_here;
                    fuel_cost += price_getter
                        .get_fuel_price(&navigate_connection.start_symbol)
                        .await?
                        * fuel_needed_to_purchase_here;
                }
            }
        }
    }

    Ok(Route {
        connections: simple,
        total_distance: distance,
        travel_time,
        total_travel_time: travel_time + (jump_cooldown - last_jump_cooldown),
        total_jump_cooldown_time: jump_cooldown,
        total_api_requests: api_requests,

        total_fuel: total_fuel_needed,
        total_refuel: total_fuel_items_needed,
        total_fuel_cost: fuel_cost,

        total_anti_matter: antimatter_needed,
        total_anti_matter_cost: antimatter_cost,
    })
}
