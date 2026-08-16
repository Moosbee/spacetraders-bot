use ::utils::WaypointCan;
use chrono::{DateTime, Utc};
use database::DatabaseConnectorAsync;
use database::TransactionReason;
use space_traders_client::models;

use std::fmt::Debug;

mod connection;
mod jump_gate_router;
mod nav_mode;
mod navigator;
mod pathfinder;
mod system_router;
mod travel_price;
mod utils;

pub use connection::ConcreteConnection;
pub use connection::JumpConnection;
pub use connection::NavigateConnection;
pub use connection::Refuel;
pub use connection::Route;
pub use connection::SimpleConnection;
pub use connection::WarpConnection;
pub use connection::assemble_route;

pub use jump_gate_router::JumpGateRouter;
pub use jump_gate_router::JumpGateRouterCache;
pub use jump_gate_router::generate_all_connections;
pub use nav_mode::NavMode;
pub use pathfinder::NavigatorCache;
pub use system_router::SystemRouter;
pub use system_router::SystemRouterCache;
pub use system_router::SystemRouterConfig;
pub use travel_price::SimpleTravelPriceCalc;
pub use travel_price::TravelPriceCache;
pub use travel_price::TravelPriceCalc;

use crate::error::Result;

use super::Mutable;
use super::RustShip;

impl<T: Clone + Send + Sync> RustShip<T, Mutable> {
    pub async fn nav_to(
        &mut self,
        waypoint: &str,
        update_market: bool,
        reason: TransactionReason,
        database_pool: &database::DbPool,
        api: &space_traders_client::Api,
        update_funds_fn: impl Fn(i64) + Clone,
    ) -> Result<()> {
        self.nav_to_prepare(
            waypoint,
            update_market,
            reason,
            false,
            database_pool,
            api,
            update_funds_fn,
        )
        .await
    }

    pub async fn nav_to_prepare(
        &mut self,
        waypoint: &str,
        update_market: bool,
        reason: TransactionReason,
        prepare: bool, // prepare to have enough fuel to leave the waypoint without a marketplace
        database_pool: &database::DbPool,
        api: &space_traders_client::Api,
        update_funds_fn: impl Fn(i64) + Clone,
    ) -> Result<()> {
        let mut pathfinder = pathfinder::NavigatorCache::default();
        let found_route = pathfinder
            .get_route(
                database_pool,
                &self.nav.waypoint_symbol,
                waypoint,
                &self.get_nav_stats(),
            )
            .await?
            .ok_or(crate::error::Error::General("No route found".to_string()))?;

        let mut price_calc = travel_price::TravelPriceCache::new(database_pool.clone());

        price_calc
            .preload_system_prices(&self.nav.system_symbol)
            .await?;

        let route =
            connection::assemble_route(&found_route, &self.get_nav_stats(), &mut price_calc)
                .await?;

        let database_pool2 = database_pool.clone();
        let api2 = api.clone();
        let route2 = route.clone();
        let reson2 = reason.clone();
        let update_funds_fn2 = update_funds_fn.clone();
        let wp_action = async move |shipi: &mut RustShip<_, Mutable>,
                                    start_waypoint: String,
                                    _end_waypoint: String| {
            let start = database::Waypoint::get_by_id(&database_pool2, &start_waypoint).await?;

            if let Some(start) = start {
                if update_market && start.is_marketplace() {
                    shipi.update_market(&api2, &database_pool2).await?;
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
                        shipi.ensure_docked(&api2).await?;
                        shipi
                            .purchase_cargo(
                                &api2,
                                &models::TradeSymbol::Fuel,
                                1,
                                &database_pool2,
                                reson2.clone(),
                                update_funds_fn2.clone(),
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
            database_pool,
            api,
            wp_action,
            update_funds_fn,
        )
        .await?;

        Ok(())
    }
}

impl<T: Clone + Send + Sync, State: Send + Sync> RustShip<T, State> {
    pub fn get_nav_stats(&self) -> ShipNavStats {
        ShipNavStats {
            can_warp: self.modules.modules.iter().any(|module| {
                module == &models::ship_module::Symbol::WarpDriveI
                    || module == &models::ship_module::Symbol::WarpDriveIi
                    || module == &models::ship_module::Symbol::WarpDriveIii
            }),
            engine_condition: self.conditions.engine.condition,
            engine_speed: self.engine_speed,
            max_cargo: self.cargo.capacity as u32,
            max_fuel: self.fuel.capacity as u32,
            only_markets: true,
            start_range: Some(
                (self.fuel.current as u32
                    + self.cargo.get_amount(&models::TradeSymbol::Fuel) as u32)
                    .min(self.fuel.capacity as u32),
            ),
            nav_mode: nav_mode::NavMode::BurnAndCruiseAndDrift,
        }
    }
}

#[derive(Clone, Default, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "ShipAutopilotState")]
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
    #[graphql(skip)]
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

#[derive(Debug, Clone, async_graphql::InputObject)]
pub struct ShipNavStats {
    pub max_fuel: u32,
    pub max_cargo: u32,
    pub start_range: Option<u32>,
    pub only_markets: bool,
    pub can_warp: bool,
    pub engine_speed: i32,
    pub engine_condition: f64,
    pub nav_mode: nav_mode::NavMode,
}

impl Default for ShipNavStats {
    fn default() -> Self {
        Self {
            // default ship is command ship
            max_fuel: 400,
            max_cargo: 40,
            start_range: None,
            only_markets: true,
            can_warp: false,
            engine_speed: 36,
            engine_condition: 1.0,
            nav_mode: NavMode::BurnAndCruiseAndDrift,
        }
    }
}

impl std::hash::Hash for ShipNavStats {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.max_fuel.hash(state);
        self.max_cargo.hash(state);
        self.start_range.hash(state);
        self.only_markets.hash(state);
        self.can_warp.hash(state);
        self.engine_speed.hash(state);
        // self.engine_condition.hash(state);
        self.nav_mode.hash(state);
    }
}

impl PartialEq for ShipNavStats {
    fn eq(&self, other: &Self) -> bool {
        self.max_fuel == other.max_fuel
            && self.max_cargo == other.max_cargo
            && self.start_range == other.start_range
            && self.only_markets == other.only_markets
            && self.can_warp == other.can_warp
            && self.engine_speed == other.engine_speed
            // && self.engine_condition == other.engine_condition
            && self.nav_mode == other.nav_mode
    }
}

impl Eq for ShipNavStats {}
