use crate::{
    error::{self, Result},
    sql,
};
use chrono::{DateTime, TimeDelta, Utc};
use log::{debug, warn};
use space_traders_client::{apis, models};
use std::collections::HashMap;

use crate::{
    api,
    ship::ship_models::MyShip,
    sql::{DatabaseConnector, TransactionReason},
};

use super::nav_models::{AutopilotState, NavigationState, RouteInstruction};

impl MyShip {
    pub async fn nav_to(
        &mut self,
        waypoint: &str,
        update_market: bool,
        waypoints: &HashMap<String, sql::Waypoint>,
        api: &api::Api,
        database_pool: crate::sql::DbPool,
        reason: TransactionReason,
    ) -> Result<()> {
        self.mutate();
        self.nav_to_prepare(
            waypoint,
            update_market,
            waypoints,
            api,
            database_pool,
            reason,
            false,
        )
        .await
    }

    pub async fn nav_to_prepare(
        &mut self,
        waypoint: &str,
        update_market: bool,
        waypoints: &HashMap<String, sql::Waypoint>,
        api: &api::Api,
        database_pool: crate::sql::DbPool,
        reason: TransactionReason,
        prepare: bool, // prepare to have enough fuel to leave the waypoint without a marketplace
    ) -> Result<()> {
        self.mutate();
        let route: Vec<super::nav_models::RouteConnection> =
            self.calculate_route(waypoints, waypoint)?;
        let route_stats: (Vec<super::nav_models::ConnectionDetails>, f64, i32, f64) =
            super::stats::calc_route_stats(
                waypoints,
                &route,
                self.engine_speed,
                self.conditions.engine.condition,
                self.conditions.frame.condition,
                self.conditions.reactor.condition,
            );
        let instructions: Vec<RouteInstruction> =
            super::stats::generate_route_instructions(route_stats.0.clone(), prepare);

        let current = Utc::now();
        let arrival = current + TimeDelta::seconds(route_stats.3.round() as i64);

        self.nav.auto_pilot = Some(AutopilotState {
            departure_time: current,
            destination_symbol: waypoint.to_string(),
            destination_system_symbol: waypoints.get(waypoint).unwrap().system_symbol.clone(),
            distance: route_stats.1,
            fuel_cost: route_stats.2,
            origin_symbol: self.nav.waypoint_symbol.clone(),
            origin_system_symbol: self.nav.system_symbol.clone(),
            instructions: instructions.clone(),
            travel_time: route_stats.3,
            arrival,
            connections: route_stats.0.clone(),
        });
        self.notify().await;

        for instruction in instructions.iter().zip(route_stats.0.iter()) {
            self.execute_navigation_step(
                instruction.0.clone(),
                api,
                &database_pool,
                update_market,
                reason.clone(),
            )
            .await?;

            debug!(
                "Travel Time {} {} {} {:?} {} {:?} {}",
                self.symbol,
                self.nav.route.departure_time,
                self.nav.route.arrival,
                (self.nav.route.arrival - self.nav.route.departure_time)
                    .abs()
                    .to_std(),
                instruction.1.travel_time,
                self.conditions,
                self.cargo.units
            );
        }

        let _ = self.wait_for_arrival_mut(api).await;
        self.nav.refresh_nav();
        self.nav.auto_pilot = None;
        self.notify().await;

        Ok(())
    }

    /// Calculate a route from the ship's current waypoint to the given waypoint.
    ///
    /// This function uses the ship's fuel capacity and current fuel levels to
    /// determine how much fuel to use when calculating the route. It will use
    /// the minimum of the ship's fuel capacity and the current fuel levels, minus
    /// any fuel that is already allocated to the ship's cargo.
    ///
    /// The route is calculated using the `find_route` method, and the mode is set
    /// to `BurnAndCruiseAndDrift`. The route is then returned as a `Vec` of
    /// `RouteConnection`s.
    ///
    pub fn calculate_route(
        &mut self,
        waypoints: &HashMap<String, sql::Waypoint>,
        waypoint: &str,
    ) -> Result<Vec<super::nav_models::RouteConnection>> {
        let start_range: i32 = self.fuel.capacity.min(
            self.fuel.current
                + (self
                    .cargo
                    .get_amount(&space_traders_client::models::TradeSymbol::Fuel)
                    * 100),
        );

        let start_range = if self.fuel.capacity == 0 {
            i32::MAX
        } else {
            start_range
        };

        let route = self.find_route(
            waypoints,
            self.nav.waypoint_symbol.clone(),
            waypoint.to_string(),
            &super::nav_models::NavMode::BurnAndCruiseAndDrift,
            true,
            start_range,
        )?;
        Ok(route)
    }

    /// Execute a single navigation step.
    ///
    /// This function will validate the ship is currently at the correct waypoint,
    /// wait for the ship to arrive at the next waypoint, handle any refueling
    /// required, update the flight mode, ensure the ship is undocked and then
    /// execute the navigation to the next waypoint, saving the route data to
    /// the database.
    ///
    /// This function will return an error if the ship is not at the correct
    /// waypoint, if the ship is unable to refuel or if the ship is unable to
    /// navigate to the next waypoint.
    ///
    async fn execute_navigation_step(
        &mut self,
        instruction: RouteInstruction,
        api: &api::Api,
        database_pool: &crate::sql::DbPool,
        update_market: bool,
        reason: TransactionReason,
    ) -> Result<()> {
        self.mutate();
        self.validate_current_waypoint(&instruction)?;
        let _ = self.wait_for_arrival_mut(api).await;

        let _erg = self
            .handle_refueling(
                &instruction,
                api,
                database_pool,
                update_market,
                reason.clone(),
            )
            .await?;
        // tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        self.mutate();

        self.update_flight_mode(api, instruction.flight_mode)
            .await?;
        self.ensure_undocked(api).await?;

        debug!(
            "Navigating from {} to {} waiting",
            self.nav.waypoint_symbol, instruction.end_symbol
        );

        let start_id = self.snapshot(database_pool).await?;

        let nav_data = self.navigate(api, &instruction.end_symbol).await?;

        let end_id = self.snapshot(database_pool).await?;

        if !nav_data.data.events.is_empty() {
            debug!("Nav Events: {:#?} ", nav_data.data.events);
        }

        let rote = crate::sql::Route {
            id: 0,
            ship_symbol: self.symbol.clone(),
            from: self.nav.waypoint_symbol.clone(),
            to: instruction.end_symbol.clone(),
            nav_mode: self.nav.flight_mode.to_string(),
            distance: instruction.distance,
            fuel_cost: nav_data.data.fuel.consumed.map(|f| f.amount).unwrap_or(0),
            travel_time: (self.nav.route.arrival - self.nav.route.departure_time).num_milliseconds()
                as f64
                / 1000.0,
            ship_info_before: Some(start_id),
            ship_info_after: Some(end_id),
            created_at: Utc::now().naive_utc(),
        };

        crate::sql::Route::insert(database_pool, &rote).await?;

        Ok(())
    }

    fn validate_current_waypoint(&self, instruction: &RouteInstruction) -> Result<()> {
        if instruction.start_symbol != self.nav.waypoint_symbol {
            return Err(error::Error::General(format!(
                "Not on waypoint {} {}",
                self.nav.waypoint_symbol, instruction.start_symbol
            )));
        }
        Ok(())
    }
}
