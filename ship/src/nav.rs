use chrono::{DateTime, Utc};
use tracing::{debug, warn, error};
use space_traders_client::{apis, models};

use crate::error;

use super::{RustShip, autopilot::AutopilotState};

use std::fmt::Debug;

// mod cache;
// mod nav_mode;
// mod pathfinding;
// mod tests;
// mod utils;

// pub mod nav_models;
// pub mod navigation;
// pub mod stats;

#[derive(Default, serde::Serialize, Clone, async_graphql::SimpleObject)]
pub struct NavigationState {
    pub flight_mode: models::ShipNavFlightMode,
    pub status: models::ShipNavStatus,
    pub system_symbol: String,
    pub waypoint_symbol: String,
    pub route: RouteState,
    pub auto_pilot: Option<AutopilotState>,
}

impl Debug for NavigationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationState")
            .field("flight_mode", &self.flight_mode)
            .field("status", &self.status)
            .field("system_symbol", &self.system_symbol)
            .field("waypoint_symbol", &self.waypoint_symbol)
            .field("route", &self.route)
            // .field("auto_pilot", &self.auto_pilot)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default, serde::Serialize, Clone, async_graphql::SimpleObject)]
pub struct RouteState {
    pub arrival: DateTime<Utc>,
    pub departure_time: DateTime<Utc>,
    pub destination_symbol: String,
    pub destination_system_symbol: String,
    pub origin_symbol: String,
    pub origin_system_symbol: String,
}

impl<T: Clone + Send + Sync + async_graphql::OutputType> RustShip<T> {
    pub async fn navigate(
        &mut self,
        api: &space_traders_client::Api,
        waypoint_symbol: &str,
    ) -> error::Result<models::NavigateShip200Response> {
        self.mutate();
        let nav_data = api
            .navigate_ship(
                &self.symbol,
                models::NavigateShipRequest {
                    waypoint_symbol: waypoint_symbol.to_string(),
                },
            )
            .await?;

        self.fuel.update(&nav_data.data.fuel);
        self.nav.update(&nav_data.data.nav);

        self.notify().await;

        core::result::Result::Ok(nav_data)
    }

    pub async fn jump(
        &mut self,
        api: &space_traders_client::Api,
        waypoint_symbol: &str,
    ) -> error::Result<models::JumpShip200Response> {
        self.mutate();
        let jump_data = api
            .jump_ship(
                &self.symbol,
                models::JumpShipRequest {
                    waypoint_symbol: waypoint_symbol.to_string(),
                },
            )
            .await?;

        self.nav.update(&jump_data.data.nav);
        self.update_cooldown(&jump_data.data.cooldown);

        self.notify().await;

        Ok(jump_data)
    }

    pub async fn warp(
        &mut self,
        api: &space_traders_client::Api,
        waypoint_symbol: &str,
    ) -> error::Result<models::NavigateShip200Response> {
        self.mutate();
        let warp_data = api
            .warp_ship(
                &self.symbol,
                models::NavigateShipRequest {
                    waypoint_symbol: waypoint_symbol.to_string(),
                },
            )
            .await?;

        self.nav.update(&warp_data.data.nav);
        self.fuel.update(&warp_data.data.fuel);
        self.notify().await;

        core::result::Result::Ok(warp_data)
    }

    async fn dock(
        &mut self,
        api: &space_traders_client::Api,
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
        api: &space_traders_client::Api,
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
        api: &space_traders_client::Api,
    ) -> core::result::Result<(), apis::Error<apis::fleet_api::DockShipError>> {
        self.mutate();
        if self.nav.get_status() != models::ShipNavStatus::Docked {
            self.dock(api).await?;
        }
        core::result::Result::Ok(())
    }

    pub async fn ensure_undocked(
        &mut self,
        api: &space_traders_client::Api,
    ) -> core::result::Result<(), apis::Error<apis::fleet_api::OrbitShipError>> {
        self.mutate();
        if self.nav.get_status() == models::ShipNavStatus::Docked {
            self.undock(api).await?;
        }
        core::result::Result::Ok(())
    }

    // async fn patch_ship_nav(
    //     &mut self,
    //     api: &space_traders_client::Api,
    //     flight_mode: models::ShipNavFlightMode,
    // ) -> core::result::Result<
    //     models::PatchShipNav200Response,
    //     apis::Error<apis::fleet_api::PatchShipNavError>,
    // > {
    //     self.mutate();
    //     let ship_patch_data = api
    //         .patch_ship_nav(
    //             &self.symbol,
    //             Some(models::PatchShipNavRequest {
    //                 flight_mode: Some(flight_mode),
    //             }),
    //         )
    //         .await?;
    //     self.nav.update(&ship_patch_data.data.nav);
    //     self.fuel.update(&ship_patch_data.data.fuel);
    //     self.notify().await;
    //     core::result::Result::Ok(ship_patch_data)
    // }

    async fn patch_ship_nav(
        &mut self,
        api: &space_traders_client::Api,
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
                        error!("Slow down while in transit");
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
        api: &space_traders_client::Api,
        flight_mode: models::ShipNavFlightMode,
    ) -> core::result::Result<(), apis::Error<apis::fleet_api::PatchShipNavError>> {
        self.mutate();
        if flight_mode != self.nav.flight_mode {
            debug!(flight_mode = ?flight_mode, "Changing flight mode");

            let current_fuel = self.fuel.current;

            let erg = self.patch_ship_nav(api, flight_mode).await?;

            if !erg.data.events.is_empty() {
                debug!(events = ?erg.data.events, "Patch Nav Events");
            }

            if erg.data.fuel.current != current_fuel {
                warn!(
                    current_fuel = %current_fuel,
                    new_fuel = %erg.data.fuel.current,
                    consumed = ?erg.data.fuel.consumed,
                    "Fuel changed after changing flight mode"
                );
            }
        }
        core::result::Result::Ok(())
    }

    pub async fn refuel_ship(
        &mut self,
        api: &space_traders_client::Api,
        units: i32, // fuel units, not cargo units, 1 cargo unit = 100 fuel
        from_cargo: bool,
    ) -> error::Result<models::RefuelShip200Response> {
        self.mutate();

        let refuel_data: models::RefuelShip200Response = api
            .refuel_ship(
                &self.symbol,
                Some(space_traders_client::models::RefuelShipRequest {
                    from_cargo: Some(from_cargo),
                    units: Some(units),
                }),
            )
            .await?;

        self.fuel.update(&refuel_data.data.fuel);

        if let Some(cargo) = refuel_data.data.cargo.as_ref() {
            self.cargo.update(cargo);
        }
        self.notify().await;

        Ok(refuel_data)
    }

    pub async fn wait_for_arrival_mut(
        &mut self,
        _api: &space_traders_client::Api,
    ) -> anyhow::Result<()> {
        let t = self.nav.route.arrival - Utc::now();
        let t = t.num_milliseconds();
        if t < 0 {
            return Ok(());
        }
        let t = t.try_into()?;
        tokio::time::sleep(tokio::time::Duration::from_millis(t)).await;
        Ok(())
    }

    pub async fn wait_for_arrival(&self) {
        let t = self.nav.route.arrival - Utc::now();
        let t = t.num_milliseconds();
        if t < 0 {
            return;
        }
        let t = t.try_into().unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(t)).await;
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

    pub(crate) fn refresh_nav(&mut self) {
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
