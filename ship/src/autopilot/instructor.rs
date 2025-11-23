use super::{
    SimpleConnection,
    connection::{
        ConcreteConnection, ConnectionType, JumpConnection, NavigateConnection, Refuel, Route,
        WarpConnection,
    },
    stats::{calculate_jump_cooldown, get_travel_stats},
};
use crate::{RustShip, error::Result};

impl<T: Clone + Send + Sync> RustShip<T> {
    pub async fn assemble_route(&self, connections: &[SimpleConnection]) -> Result<Route> {
        let simple = self.to_connection(connections);

        let mut distance = 0.0;
        let mut travel_time = 0.0;
        let mut fuel_cost = 0.0;
        let mut api_requests = 0;
        let mut jump_cooldown = 0.0;
        let mut last_jump_cooldown = 0.0;

        for c in simple.iter() {
            match c {
                ConcreteConnection::JumpGate(jump_connection) => {
                    distance += jump_connection.distance;
                    fuel_cost += 6_000.0;
                    api_requests += 1;
                    jump_cooldown += jump_connection.cooldown_time;
                    last_jump_cooldown = jump_cooldown;
                }
                ConcreteConnection::Warp(warp_connection) => {
                    distance += warp_connection.distance;
                    travel_time += warp_connection.travel_time + 1.0;
                    fuel_cost += (warp_connection.refuel.fuel_needed as f64 / 100.0).ceil();
                    api_requests += 4;
                }
                ConcreteConnection::Navigate(navigate_connection) => {
                    distance += navigate_connection.distance;
                    travel_time += navigate_connection.travel_time + 1.0;
                    fuel_cost += (navigate_connection.refuel.fuel_needed as f64 / 100.0).ceil();
                    api_requests += 4;
                }
            }
        }

        Ok(Route {
            connections: simple,
            total_distance: distance,
            total_fuel_cost: fuel_cost,
            total_travel_time: travel_time + (jump_cooldown - last_jump_cooldown),
            total_api_requests: api_requests,
        })
    }
    pub fn assemble_simple_route(
        &self,
        connections: &[SimpleConnection],
        fuel_price: i32,
        antimatter_price: i32,
    ) -> Result<Route> {
        let simple = self.to_connection(connections);

        let mut big_stats = (0.0, 0.0, 0.0, 0);

        for c in simple.iter() {
            match c {
                ConcreteConnection::JumpGate(jump_connection) => {
                    big_stats.0 += jump_connection.distance;
                    big_stats.2 += antimatter_price as f64;
                    big_stats.3 += 1;
                }
                ConcreteConnection::Warp(warp_connection) => {
                    big_stats.0 += warp_connection.distance;
                    big_stats.1 += warp_connection.travel_time;
                    big_stats.2 += (warp_connection.refuel.fuel_needed as f64 / 100.0).ceil()
                        * (fuel_price as f64);
                    big_stats.3 += 4;
                }
                ConcreteConnection::Navigate(navigate_connection) => {
                    big_stats.0 += navigate_connection.distance;
                    big_stats.1 += navigate_connection.travel_time;
                    big_stats.2 += (navigate_connection.refuel.fuel_needed as f64 / 100.0).ceil()
                        * (fuel_price as f64);
                    big_stats.3 += 4;
                }
            }
        }

        Ok(Route {
            connections: simple,
            total_distance: big_stats.0,
            total_fuel_cost: big_stats.2,
            total_travel_time: big_stats.1,
            total_api_requests: big_stats.3,
        })
    }

    pub fn to_connection(&self, connections: &[SimpleConnection]) -> Vec<ConcreteConnection> {
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
                        self.engine_speed,
                        nav_mode,
                        self.conditions.engine.condition,
                        self.conditions.frame.condition,
                        self.conditions.reactor.condition,
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
                        self.engine_speed,
                        nav_mode,
                        self.conditions.engine.condition,
                        self.conditions.frame.condition,
                        self.conditions.reactor.condition,
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
}
