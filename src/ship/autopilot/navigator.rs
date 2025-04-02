use chrono::Utc;
use log::{debug, warn};
use space_traders_client::models::{self};

use crate::{
    api,
    ship::MyShip,
    sql::{self, DatabaseConnector},
};

use super::connection::{
    ConcreteConnection, JumpConnection, NavigateConnection, Refuel, Route, WarpConnection,
};

impl MyShip {
    pub async fn fly_route(
        &mut self,
        route: Route,
        reason: sql::TransactionReason,
        database_pool: &sql::DbPool,
        api: &api::Api,
        wp_action: impl AsyncFn(&mut MyShip, String, String) -> crate::error::Result<()> + Clone,
    ) -> crate::error::Result<()> {
        for connection in route.connections {
            self.execute_connection(connection, &reason, database_pool, api, wp_action.clone())
                .await?;
        }
        self.wait_for_arrival().await;
        self.nav.refresh_nav();
        self.notify().await;
        Ok(())
    }

    async fn execute_connection(
        &mut self,
        connection: ConcreteConnection,
        reason: &sql::TransactionReason,
        database_pool: &sql::DbPool,
        api: &api::Api,
        wp_action: impl AsyncFn(&mut MyShip, String, String) -> crate::error::Result<()>,
    ) -> crate::error::Result<()> {
        match connection {
            ConcreteConnection::JumpGate(jump_connection) => {
                self.execute_jump_connection(
                    jump_connection,
                    reason,
                    database_pool,
                    api,
                    wp_action,
                )
                .await?;
            }
            ConcreteConnection::Warp(warp_connection) => {
                self.execute_warp_connection(
                    warp_connection,
                    reason,
                    database_pool,
                    api,
                    wp_action,
                )
                .await?;
            }
            ConcreteConnection::Navigate(navigate_connection) => {
                self.execute_navigate_connection(
                    navigate_connection,
                    reason,
                    database_pool,
                    api,
                    wp_action,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn execute_jump_connection(
        &mut self,
        connection: JumpConnection,
        reason: &sql::TransactionReason,
        database_pool: &sql::DbPool,
        api: &api::Api,
        wp_action: impl (AsyncFn(&mut MyShip, String, String) -> crate::error::Result<()>),
    ) -> crate::error::Result<()> {
        if self.nav.waypoint_symbol != connection.start_symbol {
            return Err("Not on waypoint".into());
        }
        if connection.start_symbol == connection.end_symbol {
            return Ok(());
        }

        wp_action(
            self,
            connection.start_symbol.clone(),
            connection.end_symbol.clone(),
        )
        .await?;

        let jump_conn =
            sql::JumpGateConnection::get_all_from(database_pool, &connection.start_symbol).await?;

        if !jump_conn.iter().any(|jg| jg.to == connection.end_symbol) {
            return Err("No connection".into());
        }
        self.wait_for_cooldown().await;

        self.ensure_undocked(api).await?;

        let jump_data = self.jump(api, &connection.end_symbol).await?;

        sql::Agent::insert(
            database_pool,
            &sql::Agent::from((*jump_data.data.agent).clone()),
        )
        .await?;

        let transaction =
            sql::MarketTransaction::try_from(jump_data.data.transaction.as_ref().clone())?
                .with(reason.clone());
        sql::MarketTransaction::insert(database_pool, &transaction).await?;

        Ok(())
    }

    async fn execute_warp_connection(
        &mut self,
        connection: WarpConnection,
        reason: &sql::TransactionReason,
        database_pool: &sql::DbPool,
        api: &api::Api,
        wp_action: impl AsyncFn(&mut MyShip, String, String) -> crate::error::Result<()>,
    ) -> crate::error::Result<()> {
        if self.nav.waypoint_symbol != connection.start_symbol {
            return Err("Not on waypoint".into());
        }
        if connection.start_symbol == connection.end_symbol {
            return Ok(());
        }

        self.wait_for_arrival().await;

        self.handle_refuel(connection.refuel, reason, database_pool, api)
            .await?;

        self.update_flight_mode(api, connection.nav_mode).await?;

        wp_action(
            self,
            connection.start_symbol.clone(),
            connection.end_symbol.clone(),
        )
        .await?;

        self.ensure_undocked(api).await?;

        debug!(
            "Navigating from {} to {} waiting",
            self.nav.waypoint_symbol, connection.end_symbol
        );

        let start_id = self.snapshot(database_pool).await?;

        let nav_data = self.warp(api, &connection.end_symbol).await?;

        let end_id = self.snapshot(database_pool).await?;

        let rote = crate::sql::Route {
            id: 0,
            ship_symbol: self.symbol.clone(),
            from: self.nav.waypoint_symbol.clone(),
            to: connection.end_symbol.clone(),
            nav_mode: self.nav.flight_mode.to_string(),
            distance: connection.distance,
            fuel_cost: nav_data.data.fuel.consumed.map(|f| f.amount).unwrap_or(0),
            travel_time: ((self.nav.route.arrival - self.nav.route.departure_time)
                .num_milliseconds() as f64)
                / 1000.0,
            ship_info_before: Some(start_id),
            ship_info_after: Some(end_id),
            created_at: Utc::now(),
        };

        crate::sql::Route::insert(database_pool, &rote).await?;

        Ok(())
    }

    async fn execute_navigate_connection(
        &mut self,
        connection: NavigateConnection,
        reason: &sql::TransactionReason,
        database_pool: &sql::DbPool,
        api: &api::Api,
        wp_action: impl AsyncFn(&mut MyShip, String, String) -> crate::error::Result<()>,
    ) -> crate::error::Result<()> {
        if self.nav.waypoint_symbol != connection.start_symbol {
            return Err("Not on waypoint".into());
        }
        if connection.start_symbol == connection.end_symbol {
            return Ok(());
        }

        self.wait_for_arrival().await;

        self.handle_refuel(connection.refuel, reason, database_pool, api)
            .await?;

        self.update_flight_mode(api, connection.nav_mode).await?;

        wp_action(
            self,
            connection.start_symbol.clone(),
            connection.end_symbol.clone(),
        )
        .await?;

        self.ensure_undocked(api).await?;

        debug!(
            "Navigating from {} to {} waiting",
            self.nav.waypoint_symbol, connection.end_symbol
        );

        let start_id = self.snapshot(database_pool).await?;

        let nav_data = self.navigate(api, &connection.end_symbol).await?;

        let end_id = self.snapshot(database_pool).await?;

        if !nav_data.data.events.is_empty() {
            debug!("Nav Events: {:#?} ", nav_data.data.events);
        }

        let rote = crate::sql::Route {
            id: 0,
            ship_symbol: self.symbol.clone(),
            from: self.nav.waypoint_symbol.clone(),
            to: connection.end_symbol.clone(),
            nav_mode: self.nav.flight_mode.to_string(),
            distance: connection.distance,
            fuel_cost: nav_data.data.fuel.consumed.map(|f| f.amount).unwrap_or(0),
            travel_time: ((self.nav.route.arrival - self.nav.route.departure_time)
                .num_milliseconds() as f64)
                / 1000.0,
            ship_info_before: Some(start_id),
            ship_info_after: Some(end_id),
            created_at: Utc::now(),
        };

        crate::sql::Route::insert(database_pool, &rote).await?;

        Ok(())
    }

    async fn handle_refuel(
        &mut self,
        refuel: Refuel,
        reason: &sql::TransactionReason,
        database_pool: &sql::DbPool,
        api: &api::Api,
    ) -> crate::error::Result<()> {
        // the system should refuel at least to such a level, that we can navigate.
        // the system should refuel as much as would fit in the fuel tanks. But not waste fuel by overfilling the tanks. Except overfilling is required for navigation.
        // if the start is a marketplace we should dock, refuel and buy necessary fuel for the cargo hold.
        // if the start has no marketplace we will be refueling from the cargo hold.
        // in both cases the amount of fuel refueled is the max fuel without overfilling the tanks that get's us to navigate.

        let is_marketplace = refuel.start_is_marketplace;

        let requirements = self.calculate_refuelments(refuel);

        if !requirements.needs_refuel() {
            return Ok(());
        }

        if is_marketplace {
            self.marketplace_refuel(requirements, api, reason, database_pool)
                .await?;
        } else {
            self.space_refuel(requirements, api, reason, database_pool)
                .await?;
        }
        Ok(())
    }

    fn calculate_refuelments(&self, refuel: Refuel) -> RefuelRequirements {
        debug!("Calculating refuel requirements: {:?}", refuel);

        if self.fuel.capacity == 0 {
            return RefuelRequirements {
                refuel_amount: 0,
                restock_amount: 0,
            };
        }

        // the system should refuel at least to such a level, that we can navigate.
        // the system should refuel as much as would fit in the fuel tanks. But not waste fuel by overfilling the tanks. Except overfilling is required for navigation.
        // if the start is a marketplace we should dock, refuel and buy necessary fuel for the cargo hold.
        // if the start has no marketplace we will be refueling from the cargo hold.
        // in both cases the amount of fuel refueled is the max fuel without overfilling the tanks that get's us to navigate.

        let current_fuel = self.fuel.current;
        let max_fuel = self.fuel.capacity;
        let min_refuel_to = refuel.fuel_needed;

        let refuel_to_min =
            ((((min_refuel_to - current_fuel).max(0) as f64) / 100.0).ceil() as i32) * 100;
        let extra_fuel =
            ((((refuel_to_min - max_fuel).max(0) as f64) / 100.0).floor() as i32) * 100;
        let refuel_to = (refuel_to_min + extra_fuel).min(max_fuel);

        let cargo_fuel = self.cargo.get_amount(&models::TradeSymbol::Fuel);
        let needed_cargo_fuel = refuel.fuel_required;

        RefuelRequirements {
            refuel_amount: refuel_to,
            restock_amount: (needed_cargo_fuel - cargo_fuel).max(0),
        }
    }

    async fn marketplace_refuel(
        &mut self,
        refuel: RefuelRequirements,
        api: &api::Api,
        reason: &sql::TransactionReason,
        database_pool: &sql::DbPool,
    ) -> crate::error::Result<()> {
        // we need to refuel/restock something

        self.ensure_docked(api).await?;

        if refuel.refuel_amount > 0 {
            let refuel_data = self.refuel_ship(api, refuel.refuel_amount, false).await?;

            sql::Agent::insert(database_pool, &sql::Agent::from(*refuel_data.data.agent)).await?;

            let transaction =
                sql::MarketTransaction::try_from(refuel_data.data.transaction.as_ref().clone())?
                    .with(reason.clone());
            sql::MarketTransaction::insert(database_pool, &transaction).await?;
        }

        if refuel.restock_amount > 0 {
            self.purchase_cargo(
                api,
                &space_traders_client::models::TradeSymbol::Fuel,
                refuel.restock_amount,
                database_pool,
                reason.clone(),
            )
            .await?;
        }

        Ok(())
    }

    async fn space_refuel(
        &mut self,
        refuel: RefuelRequirements,
        api: &api::Api,
        reason: &sql::TransactionReason,
        database_pool: &sql::DbPool,
    ) -> crate::error::Result<()> {
        if refuel.refuel_amount > 0 {
            let refuel_data = self.refuel_ship(api, refuel.refuel_amount, true).await?;

            sql::Agent::insert(database_pool, &sql::Agent::from(*refuel_data.data.agent)).await?;

            let transaction =
                sql::MarketTransaction::try_from(refuel_data.data.transaction.as_ref().clone())?
                    .with(reason.clone());
            sql::MarketTransaction::insert(database_pool, &transaction).await?;
        }

        if refuel.restock_amount > 0 {
            warn!("Cannot purchase cargo in space");
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RefuelRequirements {
    /// the fuel units to refuel
    refuel_amount: i32,
    /// the fuel storage units to buy
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
