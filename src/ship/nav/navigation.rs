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
        waypoints: &HashMap<String, models::Waypoint>,
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
        waypoints: &HashMap<String, models::Waypoint>,
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

        let _ = self.wait_for_arrival(api).await;
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
        waypoints: &HashMap<String, models::Waypoint>,
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
        let _ = self.wait_for_arrival(api).await;

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

        if nav_data.data.events.len() > 0 {
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

impl MyShip {
    async fn navigate(
        &mut self,
        api: &api::Api,
        waypoint_symbol: &str,
    ) -> error::Result<models::NavigateShip200Response> {
        self.mutate();
        let nav_data = api
            .navigate_ship(
                &self.symbol,
                Some(models::NavigateShipRequest {
                    waypoint_symbol: waypoint_symbol.to_string(),
                }),
            )
            .await?;

        self.fuel.update(&nav_data.data.fuel);
        self.nav.update(&nav_data.data.nav);

        self.notify().await;

        core::result::Result::Ok(nav_data)
    }

    pub async fn jump(
        &mut self,
        api: &api::Api,
        waypoint_symbol: &str,
        database_pool: &sql::DbPool,
        reason: sql::TransactionReason,
    ) -> error::Result<models::JumpShip200Response> {
        self.mutate();
        let jump_data = api
            .jump_ship(
                &self.symbol,
                Some(models::JumpShipRequest {
                    waypoint_symbol: waypoint_symbol.to_string(),
                }),
            )
            .await?;

        self.nav.update(&jump_data.data.nav);
        self.update_cooldown(&jump_data.data.cooldown);

        sql::Agent::insert(
            database_pool,
            &sql::Agent::from((*jump_data.data.agent).clone()),
        )
        .await?;

        let transaction =
            sql::MarketTransaction::try_from(jump_data.data.transaction.as_ref().clone())?
                .with(reason.clone());
        sql::MarketTransaction::insert(database_pool, &transaction).await?;

        self.notify().await;

        Ok(jump_data)
    }

    async fn dock(
        &mut self,
        api: &api::Api,
    ) -> core::result::Result<
        models::DockShip200Response,
        apis::Error<apis::fleet_api::DockShipError>,
    > {
        self.mutate();
        let dock_data = api.dock_ship(&self.symbol).await?;
        self.nav.update(&dock_data.data.nav);
        self.notify().await;

        core::result::Result::Ok(dock_data)
    }

    async fn undock(
        &mut self,
        api: &api::Api,
    ) -> core::result::Result<
        models::OrbitShip200Response,
        apis::Error<apis::fleet_api::OrbitShipError>,
    > {
        self.mutate();
        let undock_data: models::OrbitShip200Response = api.orbit_ship(&self.symbol).await?;
        self.nav.update(&undock_data.data.nav);
        self.notify().await;

        core::result::Result::Ok(undock_data)
    }

    pub async fn ensure_docked(
        &mut self,
        api: &api::Api,
    ) -> core::result::Result<(), apis::Error<apis::fleet_api::DockShipError>> {
        self.mutate();
        if self.nav.get_status() != models::ShipNavStatus::Docked {
            self.dock(api).await?;
        }
        core::result::Result::Ok(())
    }

    pub async fn ensure_undocked(
        &mut self,
        api: &api::Api,
    ) -> core::result::Result<(), apis::Error<apis::fleet_api::OrbitShipError>> {
        self.mutate();
        if self.nav.get_status() == models::ShipNavStatus::Docked {
            self.undock(api).await?;
        }
        core::result::Result::Ok(())
    }

    async fn patch_ship_nav(
        &mut self,
        api: &api::Api,
        flight_mode: models::ShipNavFlightMode,
    ) -> core::result::Result<
        models::PatchShipNav200Response,
        apis::Error<apis::fleet_api::PatchShipNavError>,
    > {
        self.mutate();
        let mut count = 0;
        let ship_patch_data = loop {
            let ship_patch_data_result = api
                .patch_ship_nav(
                    &self.symbol,
                    Some(models::PatchShipNavRequest {
                        flight_mode: Some(flight_mode),
                    }),
                )
                .await;

            match ship_patch_data_result {
                Ok(ship_patch_data) => break ship_patch_data,
                Err(space_traders_client::apis::Error::ResponseError(e)) => {
                    if count > 3 {
                        return core::result::Result::Err(
                            space_traders_client::apis::Error::ResponseError(e),
                        );
                    }
                    if e.status == 400 && e.content == "You can't slow down while in transit." {
                        log::error!("Slow down while in transit");
                        count += 1;
                        continue;
                    }
                }
                Err(e) => return core::result::Result::Err(e),
            }
        };
        self.nav.update(&ship_patch_data.data.nav);
        self.fuel.update(&ship_patch_data.data.fuel);
        self.notify().await;
        core::result::Result::Ok(ship_patch_data)
    }

    pub async fn update_flight_mode(
        &mut self,
        api: &api::Api,
        flight_mode: models::ShipNavFlightMode,
    ) -> core::result::Result<(), apis::Error<apis::fleet_api::PatchShipNavError>> {
        self.mutate();
        if flight_mode != self.nav.flight_mode {
            debug!("Changing flight mode to {:?}", flight_mode);

            let current_fuel = self.fuel.current;

            let erg = self.patch_ship_nav(api, flight_mode).await?;

            if erg.data.events.len() > 0 {
                debug!("Patch Nav Events: {:#?}", erg.data.events);
            }

            if erg.data.fuel.current != current_fuel {
                warn!(
                    "Fuel changed from {} to {} {:?}",
                    current_fuel, erg.data.fuel.current, erg.data.fuel.consumed
                );
            }
        }
        core::result::Result::Ok(())
    }

    pub async fn wait_for_arrival(&mut self, _api: &api::Api) -> anyhow::Result<()> {
        self.mutate();
        let t = self.nav.route.arrival - Utc::now();
        let t = t.num_milliseconds();
        if t < 0 {
            return Ok(());
        }
        let t = t.try_into()?;
        tokio::time::sleep(tokio::time::Duration::from_millis(t)).await;
        Ok(())
    }
}

impl NavigationState {
    pub fn update(&mut self, nav: &models::ShipNav) {
        self.flight_mode = nav.flight_mode;
        self.status = nav.status;
        self.system_symbol = nav.system_symbol.clone();
        self.waypoint_symbol = nav.waypoint_symbol.clone();
        self.route.arrival = DateTime::parse_from_rfc3339(&nav.route.arrival)
            .unwrap()
            .to_utc();
        self.route.departure_time = DateTime::parse_from_rfc3339(&nav.route.departure_time)
            .unwrap()
            .to_utc();
        self.route.destination_symbol = nav.route.destination.symbol.clone();
        self.route.destination_system_symbol = nav.route.destination.system_symbol.clone();
        self.route.origin_symbol = nav.route.origin.symbol.clone();
        self.route.origin_system_symbol = nav.route.origin.system_symbol.clone();
    }

    pub fn is_in_transit(&self) -> bool {
        if self.status == models::ShipNavStatus::InTransit {
            let t = self.route.arrival - Utc::now();
            let t = t.num_milliseconds();
            t > 0
        } else {
            false
        }
    }

    pub(crate) fn refresh_nav(&mut self) -> () {
        if !self.is_in_transit() && self.status == models::ShipNavStatus::InTransit {
            self.status = models::ShipNavStatus::InOrbit;
        }
    }

    pub fn get_status(&self) -> models::ShipNavStatus {
        match self.status {
            models::ShipNavStatus::Docked => models::ShipNavStatus::Docked,
            models::ShipNavStatus::InOrbit => models::ShipNavStatus::InOrbit,
            models::ShipNavStatus::InTransit => {
                if self.is_in_transit() {
                    models::ShipNavStatus::InTransit
                } else {
                    models::ShipNavStatus::InOrbit
                }
            }
        }
    }
}
