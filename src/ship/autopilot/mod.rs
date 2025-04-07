use chrono::{DateTime, Utc};
use pathfinder::Pathfinder;
use space_traders_client::models;

use crate::{
    error::Result,
    sql::{self, TransactionReason},
    types::{ConductorContext, WaypointCan},
};

use super::{modules, MyShip};

use std::fmt::Debug;

mod connection;
mod instructor;
mod nav_mode;
mod navigator;
mod pathfinder;
mod simple_pathfinding;
mod stats;
mod utils;

pub use connection::SimpleConnection;

impl MyShip {
    pub async fn nav_to(
        &mut self,
        waypoint: &str,
        update_market: bool,
        reason: TransactionReason,
        context: &ConductorContext,
    ) -> Result<()> {
        self.mutate();
        self.nav_to_prepare(waypoint, update_market, reason, false, context)
            .await
    }

    pub async fn nav_to_prepare(
        &mut self,
        waypoint: &str,
        update_market: bool,
        reason: TransactionReason,
        prepare: bool, // prepare to have enough fuel to leave the waypoint without a marketplace
        context: &ConductorContext,
    ) -> Result<()> {
        let pathfinder = self
            .get_pathfinder(context)
            .ok_or("Failed to get pathfinder")?;

        let found_route = pathfinder
            .get_route(&self.nav.waypoint_symbol, waypoint)
            .await?;

        let route = self.assemble_route(&found_route).await?;

        let context2 = context.clone();
        let route2 = route.clone();
        let reson2 = reason.clone();
        let wp_action =
            async move |shipi: &mut MyShip, start_waypoint: String, end_waypoint: String| {
                let start =
                    sql::Waypoint::get_by_symbol(&context2.database_pool, &start_waypoint).await?;

                if let Some(start) = start {
                    if update_market && start.is_marketplace() {
                        shipi
                            .update_market(&context2.api, &context2.database_pool)
                            .await?;
                    }
                    if prepare && start.is_marketplace() {
                        let mut is_last_marketplace = true;

                        for connection in route2.connections.iter().rev() {
                            match connection {
                                connection::ConcreteConnection::JumpGate(_jump_connection) => {
                                    is_last_marketplace = false;
                                    break;
                                }
                                connection::ConcreteConnection::Warp(warp_connection) => {
                                    if warp_connection.start_symbol == start_waypoint {
                                        break;
                                    }
                                    if warp_connection.end_is_marketplace
                                        || warp_connection.start_is_marketplace
                                    {
                                        is_last_marketplace = false;
                                    }
                                }
                                connection::ConcreteConnection::Navigate(navigate_connection) => {
                                    if navigate_connection.start_symbol == start_waypoint {
                                        break;
                                    }
                                    if navigate_connection.end_is_marketplace
                                        || navigate_connection.start_is_marketplace
                                    {
                                        is_last_marketplace = false;
                                    }
                                }
                            }
                        }
                        if is_last_marketplace {
                            shipi.ensure_docked(&context2.api).await?;
                            shipi
                                .purchase_cargo(
                                    &context2.api,
                                    &models::TradeSymbol::Fuel,
                                    1,
                                    &context2.database_pool,
                                    reson2.clone(),
                                )
                                .await?;
                        }
                    }
                }

                Ok(())
            };

        self.fly_route(
            route,
            reason,
            &context.database_pool,
            &context.api,
            wp_action,
        )
        .await?;

        Ok(())
    }

    pub fn get_pathfinder(&self, context: &ConductorContext) -> Option<Pathfinder> {
        Some(Pathfinder {
            range: self.fuel.capacity as u32,
            context: context.clone(),
            nav_mode: nav_mode::NavMode::BurnAndCruiseAndDrift,
            start_range: (self.fuel.current as u32
                + self.cargo.get_amount(&models::TradeSymbol::Fuel) as u32)
                .min(self.fuel.capacity as u32),
            only_markets: true,
        })
    }
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
    pub travel_time: f64,
    pub route: connection::Route,
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
