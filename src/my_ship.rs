use std::collections::HashMap;

use anyhow::Ok;
use chrono::{DateTime, Utc};
use log::{debug, info};
use space_traders_client::{
    apis,
    models::{self, PurchaseCargoRequest, WaypointTraitSymbol},
};

use crate::{
    api, path_finding,
    sql::{self, MarketTransaction, TransactionReason, With},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    Construction,
    Trader,
    Contract,
    Scraper,
    Mining,
    Manuel,
}

#[derive(Debug)]
pub struct MyShip {
    pub role: Role,
    pub registration_role: models::ShipRole,
    pub symbol: String,
    pub engine_speed: i32,
    pub cooldown_expiration: Option<DateTime<Utc>>,
    pub cargo_capacity: i32,
    pub cargo_units: i32,
    pub cargo: Vec<(models::TradeSymbol, i32)>,
    pub fuel_capacity: i32,
    pub fuel_current: i32,
    pub nav_flight_mode: models::ShipNavFlightMode,
    pub nav_status: models::ShipNavStatus,
    pub nav_system_symbol: String,
    pub nav_waypoint_symbol: String,
    pub nav_route_arrival: DateTime<Utc>,
    pub nav_route_departure_time: DateTime<Utc>,
    pub nav_route_destination_symbol: String,
    pub nav_route_destination_system_symbol: String,
    pub nav_route_origin_symbol: String,
    pub nav_route_origin_system_symbol: String,
}

impl MyShip {
    pub fn new(ship: models::Ship) -> MyShip {
        let cargo: Vec<(models::TradeSymbol, i32)> = ship
            .cargo
            .inventory
            .iter()
            .map(|f| (f.symbol, f.units))
            .collect();
        // ship.engine.condition
        // ship.engine.integrity
        MyShip {
            symbol: ship.symbol,
            role: Role::Manuel,
            engine_speed: ship.engine.speed,
            registration_role: ship.registration.role,
            cooldown_expiration: DateTime::parse_from_rfc3339(
                &ship.cooldown.expiration.unwrap_or("".to_string()),
            )
            .map(|op| op.to_utc())
            .ok(),
            cargo_capacity: ship.cargo.capacity,
            cargo_units: ship.cargo.units,
            cargo,
            fuel_capacity: ship.fuel.capacity,
            fuel_current: ship.fuel.current,
            nav_flight_mode: ship.nav.flight_mode,
            nav_status: ship.nav.status,
            nav_system_symbol: ship.nav.system_symbol,
            nav_waypoint_symbol: ship.nav.waypoint_symbol,
            nav_route_arrival: DateTime::parse_from_rfc3339(&ship.nav.route.arrival)
                .unwrap()
                .to_utc(),
            nav_route_departure_time: DateTime::parse_from_rfc3339(&ship.nav.route.departure_time)
                .unwrap()
                .to_utc(),
            nav_route_destination_symbol: ship.nav.route.destination.symbol,
            nav_route_destination_system_symbol: ship.nav.route.destination.system_symbol,
            nav_route_origin_symbol: ship.nav.route.origin.symbol,
            nav_route_origin_system_symbol: ship.nav.route.origin.system_symbol,
        }
    }
    pub fn from_ship(ship: models::Ship) -> MyShip {
        MyShip::new(ship)
    }

    fn update_fuel(&mut self, ship_fuel: &models::ShipFuel) {
        self.fuel_capacity = ship_fuel.capacity;
        self.fuel_current = ship_fuel.current;
    }

    fn update_nav(&mut self, ship_nav: &models::ShipNav) {
        self.nav_flight_mode = ship_nav.flight_mode;
        self.nav_status = ship_nav.status;
        self.nav_system_symbol = ship_nav.system_symbol.clone();
        self.nav_waypoint_symbol = ship_nav.waypoint_symbol.clone();
        self.nav_route_arrival = DateTime::parse_from_rfc3339(&ship_nav.route.arrival)
            .unwrap()
            .to_utc();
        self.nav_route_departure_time =
            DateTime::parse_from_rfc3339(&ship_nav.route.departure_time)
                .unwrap()
                .to_utc();
        self.nav_route_destination_symbol = ship_nav.route.destination.symbol.clone();
        self.nav_route_destination_system_symbol = ship_nav.route.destination.system_symbol.clone();
        self.nav_route_origin_symbol = ship_nav.route.origin.symbol.clone();
        self.nav_route_origin_system_symbol = ship_nav.route.origin.system_symbol.clone();
    }

    fn update_cargo(&mut self, ship_cargo: &models::ShipCargo) {
        self.cargo_capacity = ship_cargo.capacity;
        self.cargo_units = ship_cargo.units;
        self.cargo = ship_cargo
            .inventory
            .iter()
            .map(|f| (f.symbol, f.units))
            .collect();
    }

    pub fn get_cargo_amount(&self, symbol: &models::TradeSymbol) -> i32 {
        self.cargo
            .iter()
            .find_map(|(k, v)| if k == symbol { Some(*v) } else { None })
            .unwrap_or(0)
    }

    pub fn has_cargo(&self, symbol: &models::TradeSymbol) -> bool {
        self.get_cargo_amount(symbol) > 0
    }

    pub fn is_on_cooldown(&self) -> bool {
        if self.cooldown_expiration.is_some() {
            let t = self.cooldown_expiration.unwrap();
            let t = t - Utc::now();
            let t = t.num_seconds();
            t > 0
        } else {
            true
        }
    }

    pub async fn wait_for_cooldown(&self) -> anyhow::Result<()> {
        if self.cooldown_expiration.is_none() {
            return Err(anyhow::anyhow!("Is not on cooldown"));
        }
        let t = self.cooldown_expiration.unwrap();
        let t = t - Utc::now();
        let t = t.num_seconds().try_into()?;
        tokio::time::sleep(std::time::Duration::from_secs(t)).await;
        Ok(())
    }

    pub fn is_in_transit(&self) -> bool {
        if self.nav_status == models::ShipNavStatus::InTransit {
            let t = self.nav_route_arrival - Utc::now();
            let t = t.num_seconds();
            t > 0
        } else {
            false
        }
    }

    pub fn get_nav_status(&self) -> models::ShipNavStatus {
        match self.nav_status {
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
        let t = self.nav_route_arrival - Utc::now();
        let t = t.num_seconds().try_into()?;
        tokio::time::sleep(std::time::Duration::from_secs(t)).await;
        Ok(())
    }

    pub async fn deliver_contract(
        &mut self,
        contract_id: &str,
        trade_symbol: models::TradeSymbol,
        units: i32,
        api: &api::Api,
    ) -> anyhow::Result<models::DeliverContract200Response> {
        let delivery_result: models::DeliverContract200Response = api
            .deliver_contract(
                contract_id,
                Some(models::DeliverContractRequest {
                    units,
                    ship_symbol: self.symbol.clone(),
                    trade_symbol: trade_symbol.to_string(),
                }),
            )
            .await?;

        self.update_cargo(&delivery_result.data.cargo);

        Ok(delivery_result)
    }

    pub async fn dock(
        &mut self,
        api: &api::Api,
    ) -> Result<models::DockShip200Response, apis::Error<apis::fleet_api::DockShipError>> {
        let dock_data = api.dock_ship(&self.symbol).await?;
        self.update_nav(&dock_data.data.nav);
        core::result::Result::Ok(dock_data)
    }

    pub async fn undock(
        &mut self,
        api: &api::Api,
    ) -> anyhow::Result<models::OrbitShip200Response, apis::Error<apis::fleet_api::OrbitShipError>>
    {
        let undock_data: models::OrbitShip200Response = api.orbit_ship(&self.symbol).await?;
        self.update_nav(&undock_data.data.nav);
        core::result::Result::Ok(undock_data)
    }

    pub async fn navigate(
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

        self.update_fuel(&nav_data.data.fuel);
        self.update_nav(&nav_data.data.nav);

        core::result::Result::Ok(nav_data)
    }

    pub async fn nav_to(
        &mut self,
        waypoint: &str,
        update_market: bool,
        waypoints: &HashMap<String, models::Waypoint>,
        api: &api::Api,
        database_pool: sqlx::PgPool,
        reason: TransactionReason,
    ) -> anyhow::Result<()> {
        info!("Nav from {} to {}", self.nav_waypoint_symbol, waypoint);
        let route = path_finding::get_route_a_star(
            waypoints,
            self.nav_waypoint_symbol.clone(),
            waypoint.to_string(),
            self.fuel_capacity,
            path_finding::NavMode::BurnAndCruiseAndDrift,
            true,
        )?;

        let stats: (
            Vec<path_finding::ConnectionDetails>,
            f64,
            i32,
            chrono::TimeDelta,
        ) = path_finding::calc_route_stats(waypoints, &route, self.engine_speed);

        let instructions: Vec<RouteInstruction> = self.route_instructions(stats.0.clone());

        // info!("Instructions: {:?}", instructions);

        for inst in instructions {
            if !(inst.start_symbol == self.nav_waypoint_symbol) {
                return Err(anyhow::anyhow!(
                    "Not on waypoint {} {}",
                    self.nav_waypoint_symbol,
                    inst.start_symbol
                ));
            }

            debug!("Instruction: {:?} waiting", inst);

            let _ = self.wait_for_arrival().await;

            debug!(
                "Arrived on waypoint {} {}",
                self.nav_waypoint_symbol, inst.end_symbol
            );

            self.nav_refuel(
                &inst,
                &api,
                database_pool.clone(),
                update_market,
                reason.clone(),
            )
            .await?;

            if inst.flight_mode != self.nav_flight_mode {
                debug!("Changing flight mode to {:?}", inst.flight_mode);
                api.patch_ship_nav(
                    &self.symbol,
                    Some(models::PatchShipNavRequest {
                        flight_mode: Some(inst.flight_mode),
                    }),
                )
                .await
                .unwrap();
            }

            if self.nav_status == models::ShipNavStatus::Docked {
                self.undock(api).await?;
            }
            let start_waypoint = self.nav_waypoint_symbol.clone();

            let _nav_data = self.navigate(api, &inst.end_symbol).await?;

            info!(
                "Navigated to waypoint {} from {} arrived in {:?} being at waypoint {}",
                start_waypoint, inst.end_symbol, self.nav_route_arrival, self.nav_waypoint_symbol
            );
        }

        let _ = self.wait_for_arrival().await;

        Ok(())
    }

    // pub fn get_dijkstra(
    //     &self,
    //     waypoints: &HashMap<String, models::Waypoint>,
    // ) -> Result<HashMap<String, path_finding::RouteConnection>, anyhow::Error> {
    //     let routes = path_finding::get_full_dijkstra(
    //         waypoints,
    //         self.nav_waypoint_symbol.clone(),
    //         self.fuel_current,
    //         path_finding::NavMode::BurnAndCruiseAndDrift,
    //         true,
    //     );

    //     routes
    // }

    pub fn route_instructions(
        &self,
        route: Vec<path_finding::ConnectionDetails>,
    ) -> Vec<RouteInstruction> {
        let mut instructions = Vec::new();

        let mut last_fuel_cap = 0;

        for conn in route.iter().rev() {
            let start_is_marketplace = conn
                .start
                .traits
                .iter()
                .any(|t| t.symbol == WaypointTraitSymbol::Marketplace);

            if !start_is_marketplace {
                last_fuel_cap = last_fuel_cap + conn.fuel_cost;
            }

            instructions.push(RouteInstruction {
                start_symbol: conn.start.symbol.clone(),
                end_symbol: conn.end.symbol.clone(),
                start_is_marketplace,

                flight_mode: conn.flight_mode,
                refuel_to: conn.fuel_cost,
                fuel_in_cargo: last_fuel_cap,
            });

            if start_is_marketplace {
                last_fuel_cap = 0;
            }
        }

        instructions.reverse();

        instructions
    }
    async fn nav_refuel(
        &mut self,
        instruction: &RouteInstruction,
        api: &api::Api,
        database_pool: sqlx::PgPool,
        update_market: bool,
        reason: TransactionReason,
    ) -> anyhow::Result<()> {
        let refuel_requirements = self.calculate_refuel_requirements(instruction);

        info!("Refuel requirements: {:?}", refuel_requirements);

        if !refuel_requirements.needs_refuel() {
            return Ok(());
        }

        if instruction.start_is_marketplace {
            self.handle_marketplace_refuel(
                &api,
                &database_pool,
                refuel_requirements,
                update_market,
                reason,
            )
            .await
        } else {
            self.handle_space_refuel(&api, &database_pool, refuel_requirements, reason)
                .await
        }
    }

    fn calculate_refuel_requirements(&self, instruction: &RouteInstruction) -> RefuelRequirements {
        debug!("Calculating refuel requirements: {:?}", instruction);
        let current_fuel_stock = self
            .cargo
            .iter()
            .find(|c| c.0 == models::TradeSymbol::Fuel)
            .map_or(0, |c| c.1);

        let target_stock = ((instruction.fuel_in_cargo as f64) / 100.0).ceil() as i32;

        RefuelRequirements {
            refuel_amount: if self.fuel_current < instruction.refuel_to {
                let base_amount = instruction.refuel_to - self.fuel_current;
                ((base_amount as f64) / 100.0).ceil() as i32 * 100
            } else {
                0
            },
            restock_amount: target_stock - current_fuel_stock,
        }
    }

    async fn handle_marketplace_refuel(
        &mut self,
        api: &api::Api,
        database_pool: &sqlx::PgPool,
        requirements: RefuelRequirements,
        update_market: bool,
        reason: TransactionReason,
    ) -> anyhow::Result<()> {
        if !requirements.needs_marketplace_action() {
            return Ok(());
        }

        // Dock the ship
        self.dock(&api).await.unwrap();

        // Perform refueling if needed
        if requirements.refuel_amount > 0 {
            debug!("Marketplace refueling to fuel");
            let refuel_data = api
                .refuel_ship(
                    &self.symbol,
                    Some(models::RefuelShipRequest {
                        from_cargo: Some(false),
                        units: Some(requirements.refuel_amount),
                    }),
                )
                .await
                .unwrap();
            let transaction =
                MarketTransaction::try_from(refuel_data.data.transaction.as_ref().clone())?
                    .with(reason.clone());
            crate::sql::insert_market_transaction(&database_pool, &transaction).await;
        }

        // Restock fuel cargo if needed
        if requirements.restock_amount > 0 {
            debug!("Marketplace refueling to cargo");
            let _restock_data = self
                .purchase_cargo(
                    api,
                    models::TradeSymbol::Fuel,
                    requirements.restock_amount,
                    database_pool,
                    reason,
                )
                .await
                .unwrap();
        }

        // Return to orbit
        self.undock(&api).await.unwrap();

        // Update market data if requested
        if update_market {
            self.update_market(api, database_pool).await?;
        }

        Ok(())
    }

    pub async fn update_market(
        &self,
        api: &api::Api,
        database_pool: &sqlx::PgPool,
    ) -> anyhow::Result<()> {
        let market_data = api
            .get_market(&self.nav_system_symbol, &self.nav_waypoint_symbol)
            .await?;
        crate::workers::market_scrapers::update_market(*market_data.data, &database_pool).await;

        Ok(())
    }

    async fn handle_space_refuel(
        &mut self,
        api: &api::Api,
        database_pool: &sqlx::PgPool,
        requirements: RefuelRequirements,
        reason: TransactionReason,
    ) -> anyhow::Result<()> {
        if requirements.refuel_amount > 0 {
            debug!("Space refueling");
            let refuel_data = api
                .refuel_ship(
                    &self.symbol,
                    Some(models::RefuelShipRequest {
                        from_cargo: Some(true),
                        units: Some(requirements.refuel_amount),
                    }),
                )
                .await?;
            let transaction =
                MarketTransaction::try_from(refuel_data.data.transaction.as_ref().clone())?
                    .with(reason);
            crate::sql::insert_market_transaction(&database_pool, &transaction).await;

            self.cargo
                .iter_mut()
                .find(|c| c.0 == models::TradeSymbol::Fuel)
                .map(|c| c.1 -= requirements.refuel_amount);
        }
        Ok(())
    }

    pub async fn purchase_cargo(
        &mut self,
        api: &api::Api,
        symbol: models::TradeSymbol,
        units: i32,
        database_pool: &sqlx::PgPool,
        reason: TransactionReason,
    ) -> anyhow::Result<()> {
        let market_info =
            sql::get_last_waypoint_trade_goods(database_pool, &self.nav_waypoint_symbol).await;
        let market_info = if market_info.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            self.update_market(api, database_pool).await?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            sql::get_last_waypoint_trade_goods(database_pool, &self.nav_waypoint_symbol).await
        } else {
            market_info
        };
        let max_purchase_volume = market_info
            .iter()
            .find(|m| m.symbol == symbol)
            .map(|m| m.trade_volume)
            .ok_or(anyhow::anyhow!(
                "Could not find symbol in market info {:?}",
                market_info
            ))?;

        let full_purchases = units / max_purchase_volume;
        let last_purchase_volume = units % max_purchase_volume;

        let purchases = (0..full_purchases)
            .map(|_| max_purchase_volume)
            .chain(std::iter::once(last_purchase_volume))
            .collect::<Vec<i32>>();

        assert_eq!(purchases.iter().sum::<i32>(), units);

        for purchase_volume in purchases {
            let purchase_data = api
                .purchase_cargo(
                    &self.symbol,
                    Some(PurchaseCargoRequest {
                        symbol,
                        units: purchase_volume,
                    }),
                )
                .await
                .unwrap();
            self.update_cargo(&purchase_data.data.cargo);

            let transaction =
                MarketTransaction::try_from(purchase_data.data.transaction.as_ref().clone())?
                    .with(reason.clone());
            crate::sql::insert_market_transaction(&database_pool, &transaction).await;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct RefuelRequirements {
    refuel_amount: i32,
    restock_amount: i32,
}

impl RefuelRequirements {
    fn needs_refuel(&self) -> bool {
        self.refuel_amount > 0 || self.restock_amount > 0
    }

    fn needs_marketplace_action(&self) -> bool {
        self.refuel_amount > 0 || self.restock_amount > 0
    }
}

#[derive(Debug)]
pub struct RouteInstruction {
    start_symbol: String,
    end_symbol: String,
    flight_mode: models::ShipNavFlightMode,
    start_is_marketplace: bool,

    /// The amount of fuel that needs to be in the Tanks to do the Current jump
    refuel_to: i32,

    /// The amount of fuel in the cargo to get to the next Market
    fuel_in_cargo: i32,
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn it_works() {
    //     assert_eq!(2 + 2, 4);
    // }
}
