use anyhow::Result;
use chrono::{DateTime, TimeDelta, Utc};
use log::{debug, info};
use space_traders_client::{apis, models};
use std::collections::HashMap;

use crate::{api, ship::ship_models::MyShip, sql::TransactionReason};

use super::nav_models::{AutopilotState, NavigationState, RouteInstruction};

impl MyShip {
    pub async fn nav_to(
        &mut self,
        waypoint: &str,
        update_market: bool,
        waypoints: &HashMap<String, models::Waypoint>,
        api: &api::Api,
        database_pool: sqlx::PgPool,
        reason: TransactionReason,
    ) -> Result<()> {
        let route = self.calculate_route(waypoints, waypoint)?;
        let route_stats: (Vec<super::nav_models::ConnectionDetails>, f64, i32, f64) =
            super::stats::calc_route_stats(waypoints, &route, self.engine_speed);
        let instructions: Vec<RouteInstruction> =
            super::stats::generate_route_instructions(route_stats.0.clone());

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

            info!(
                "Time {} {} {:?} {:?}",
                self.nav.route.departure_time,
                self.nav.route.arrival,
                self.nav.route.arrival - self.nav.route.departure_time,
                instruction.1.travel_time
            );
        }

        let _ = self.nav.wait_for_arrival().await;
        self.nav.auto_pilot = None;
        self.notify().await;

        Ok(())
    }

    pub fn calculate_route(
        &mut self,
        waypoints: &HashMap<String, models::Waypoint>,
        waypoint: &str,
    ) -> Result<Vec<super::nav_models::RouteConnection>> {
        let route = self.find_route(
            waypoints,
            self.nav.waypoint_symbol.clone(),
            waypoint.to_string(),
            &super::nav_models::NavMode::BurnAndCruiseAndDrift,
            true,
        )?;
        Ok(route)
    }

    async fn execute_navigation_step(
        &mut self,
        instruction: RouteInstruction,
        api: &api::Api,
        database_pool: &sqlx::PgPool,
        update_market: bool,
        reason: TransactionReason,
    ) -> Result<()> {
        self.validate_current_waypoint(&instruction)?;
        let _ = self.nav.wait_for_arrival().await;

        self.handle_refueling(
            &instruction,
            api,
            database_pool,
            update_market,
            reason.clone(),
        )
        .await?;
        self.update_flight_mode(api, instruction.flight_mode)
            .await?;
        self.ensure_undocked(api).await?;

        debug!(
            "Navigating from {} to {} waiting",
            self.nav.waypoint_symbol, instruction.end_symbol
        );

        self.navigate(api, &instruction.end_symbol).await?;

        Ok(())
    }

    fn validate_current_waypoint(&self, instruction: &RouteInstruction) -> Result<()> {
        if instruction.start_symbol != self.nav.waypoint_symbol {
            return Err(anyhow::anyhow!(
                "Not on waypoint {} {}",
                self.nav.waypoint_symbol,
                instruction.start_symbol
            ));
        }
        Ok(())
    }
}

impl MyShip {
    async fn navigate(
        &mut self,
        api: &api::Api,
        waypoint_symbol: &str,
    ) -> anyhow::Result<models::NavigateShip200Response> {
        let nav_data = api
            .navigate_ship(
                &self.symbol,
                Some(models::NavigateShipRequest {
                    waypoint_symbol: waypoint_symbol.to_string(),
                }),
            )
            .await
            .unwrap();

        self.fuel.update(&nav_data.data.fuel);
        self.nav.update(&nav_data.data.nav);
        self.notify().await;

        core::result::Result::Ok(nav_data)
    }

    async fn dock(
        &mut self,
        api: &api::Api,
    ) -> Result<models::DockShip200Response, apis::Error<apis::fleet_api::DockShipError>> {
        let dock_data = api.dock_ship(&self.symbol).await?;
        self.nav.update(&dock_data.data.nav);
        self.notify().await;

        core::result::Result::Ok(dock_data)
    }

    async fn undock(
        &mut self,
        api: &api::Api,
    ) -> anyhow::Result<models::OrbitShip200Response, apis::Error<apis::fleet_api::OrbitShipError>>
    {
        let undock_data: models::OrbitShip200Response = api.orbit_ship(&self.symbol).await?;
        self.nav.update(&undock_data.data.nav);
        self.notify().await;

        core::result::Result::Ok(undock_data)
    }

    pub async fn ensure_docked(
        &mut self,
        api: &api::Api,
    ) -> Result<(), apis::Error<apis::fleet_api::DockShipError>> {
        if self.nav.get_status() != models::ShipNavStatus::Docked {
            self.dock(api).await?;
        }
        core::result::Result::Ok(())
    }

    pub async fn ensure_undocked(
        &mut self,
        api: &api::Api,
    ) -> anyhow::Result<(), apis::Error<apis::fleet_api::OrbitShipError>> {
        if self.nav.get_status() == models::ShipNavStatus::Docked {
            self.undock(api).await?;
        }
        core::result::Result::Ok(())
    }

    async fn patch_ship_nav(
        &mut self,
        api: &api::Api,
        flight_mode: models::ShipNavFlightMode,
    ) -> Result<models::GetShipNav200Response, apis::Error<apis::fleet_api::PatchShipNavError>>
    {
        let ship_patch_data: models::GetShipNav200Response = api
            .patch_ship_nav(
                &self.symbol,
                Some(models::PatchShipNavRequest {
                    flight_mode: Some(flight_mode),
                }),
            )
            .await?;
        self.nav.update(&ship_patch_data.data);
        self.notify().await;
        core::result::Result::Ok(ship_patch_data)
    }

    pub async fn update_flight_mode(
        &mut self,
        api: &api::Api,
        flight_mode: models::ShipNavFlightMode,
    ) -> Result<(), apis::Error<apis::fleet_api::PatchShipNavError>> {
        if flight_mode != self.nav.flight_mode {
            debug!("Changing flight mode to {:?}", flight_mode);

            self.patch_ship_nav(api, flight_mode).await?;
        }
        core::result::Result::Ok(())
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
            let t = t.num_seconds();
            t > 0
        } else {
            false
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

    pub async fn wait_for_arrival(&self) -> anyhow::Result<()> {
        let t = self.route.arrival - Utc::now();
        let t = t.num_seconds().try_into()?;
        tokio::time::sleep(std::time::Duration::from_secs(t)).await;
        Ok(())
    }
}
