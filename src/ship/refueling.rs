use log::debug;

use crate::{
    api,
    sql::{self, DatabaseConnector},
};

use super::{
    nav::nav_models::RouteInstruction,
    ship_models::{FuelState, MyShip},
};

#[derive(Debug)]
pub struct RefuelRequirements {
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

impl MyShip {
    pub async fn handle_refueling(
        &mut self,
        instruction: &RouteInstruction,
        api: &api::Api,
        database_pool: &sql::DbPool,
        update_market: bool,
        reason: sql::TransactionReason,
    ) -> anyhow::Result<()> {
        let requirements = self.calculate_refuel_requirements(instruction);

        if !requirements.needs_refuel() {
            return Ok(());
        }

        if instruction.start_is_marketplace {
            self.handle_marketplace_refuel(api, database_pool, requirements, update_market, reason)
                .await
        } else {
            self.handle_space_refuel(api, database_pool, requirements, reason)
                .await
        }
    }

    fn calculate_refuel_requirements(&self, instruction: &RouteInstruction) -> RefuelRequirements {
        debug!("Calculating refuel requirements: {:?}", instruction);
        let current_fuel_stock = self
            .cargo
            .get_amount(&space_traders_client::models::TradeSymbol::Fuel);

        let target_stock = ((instruction.fuel_in_cargo as f64) / 100.0).ceil() as i32;

        RefuelRequirements {
            refuel_amount: if self.fuel.current < instruction.refuel_to {
                let base_amount = instruction.refuel_to - self.fuel.current;
                ((base_amount as f64) / 100.0).ceil() as i32 * 100
            } else {
                0
            },
            restock_amount: target_stock - current_fuel_stock,
        }
    }

    async fn handle_space_refuel(
        &mut self,
        api: &api::Api,
        database_pool: &sql::DbPool,
        requirements: RefuelRequirements,
        reason: sql::TransactionReason,
    ) -> anyhow::Result<()> {
        if requirements.refuel_amount > 0 {
            debug!("Space refueling");
            let refuel_data = api
                .refuel_ship(
                    &self.symbol,
                    Some(space_traders_client::models::RefuelShipRequest {
                        from_cargo: Some(true),
                        units: Some(requirements.refuel_amount),
                    }),
                )
                .await?;
            self.fuel.update(&refuel_data.data.fuel);
            self.notify().await;

            sql::Agent::insert(database_pool, &sql::Agent::from(*refuel_data.data.agent)).await?;

            let transaction =
                sql::MarketTransaction::try_from(refuel_data.data.transaction.as_ref().clone())?
                    .with(reason);
            sql::MarketTransaction::insert(database_pool, &transaction).await?;

            self.cargo
                .remove_cargo(
                    &space_traders_client::models::TradeSymbol::Fuel,
                    requirements.refuel_amount,
                )
                .unwrap();
        }
        Ok(())
    }

    async fn handle_marketplace_refuel(
        &mut self,
        api: &api::Api,
        database_pool: &sql::DbPool,
        requirements: RefuelRequirements,
        update_market: bool,
        reason: sql::TransactionReason,
    ) -> anyhow::Result<()> {
        if !requirements.needs_marketplace_action() {
            return Ok(());
        }

        // Dock the ship
        self.ensure_docked(api).await.unwrap();

        // Perform refueling if needed
        if requirements.refuel_amount > 0 {
            debug!("Marketplace refueling to fuel");

            let refuel_data = api
                .refuel_ship(
                    &self.symbol,
                    Some(space_traders_client::models::RefuelShipRequest {
                        from_cargo: Some(false),
                        units: Some(requirements.refuel_amount),
                    }),
                )
                .await?;
            self.fuel.update(&refuel_data.data.fuel);
            self.notify().await;

            sql::Agent::insert(database_pool, &sql::Agent::from(*refuel_data.data.agent)).await?;

            let transaction =
                sql::MarketTransaction::try_from(refuel_data.data.transaction.as_ref().clone())?
                    .with(reason.clone());
            sql::MarketTransaction::insert(database_pool, &transaction).await?;
        }

        // Restock fuel cargo if needed
        if requirements.restock_amount > 0 {
            debug!("Marketplace refueling to cargo");
            self.purchase_cargo(
                api,
                space_traders_client::models::TradeSymbol::Fuel,
                requirements.restock_amount,
                database_pool,
                reason,
            )
            .await
            .unwrap();
        }

        // Update market data if requested
        if update_market {
            self.update_market(api, database_pool).await?;
        }

        Ok(())
    }
}

impl FuelState {
    pub fn update(&mut self, data: &space_traders_client::models::ShipFuel) {
        self.current = data.current;
        self.capacity = data.capacity;
    }
}
