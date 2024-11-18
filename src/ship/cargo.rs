use std::{i32, ops::SubAssign};

use crate::{
    api,
    sql::{self, DatabaseConnector},
};

use super::models::MyShip;

impl MyShip {
    pub async fn purchase_cargo(
        &mut self,
        api: &api::Api,
        symbol: space_traders_client::models::TradeSymbol,
        units: i32,
        database_pool: &sqlx::PgPool,
        reason: sql::TransactionReason,
    ) -> anyhow::Result<()> {
        let market_info = self.get_market_info(api, database_pool).await?;
        let purchase_volumes = self.calculate_purchase_volumes(units, &market_info, symbol)?;

        for volume in purchase_volumes {
            self.execute_purchase(api, symbol, volume, database_pool, reason.clone())
                .await?;
        }

        Ok(())
    }

    async fn get_market_info(
        &self,
        api: &api::Api,
        database_pool: &sqlx::PgPool,
    ) -> anyhow::Result<Vec<sql::MarketTradeGood>> {
        let market_info =
            sql::MarketTradeGood::get_last_by_waypoint(database_pool, &self.nav.waypoint_symbol)
                .await?;
        let market_info = if market_info.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            self.update_market(api, database_pool).await?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            sql::MarketTradeGood::get_last_by_waypoint(database_pool, &self.nav.waypoint_symbol)
                .await
        } else {
            sqlx::Result::Ok(market_info)
        };

        Ok(market_info?)
    }

    fn calculate_purchase_volumes(
        &self,
        quantity: i32,
        market_info: &[sql::MarketTradeGood],
        good: space_traders_client::models::TradeSymbol,
    ) -> anyhow::Result<Vec<i32>> {
        let max_purchase_volume = market_info
            .iter()
            .find(|m| m.symbol == good)
            .map(|m| m.trade_volume)
            .ok_or_else(|| anyhow::anyhow!("Could not find good in market info"))?;

        let mut volumes = Vec::new();
        let mut remaining = quantity;

        while remaining > 0 && max_purchase_volume > 0 {
            let volume = std::cmp::min(remaining, max_purchase_volume);
            volumes.push(volume);
            remaining -= volume;
        }

        Ok(volumes)
    }

    async fn execute_purchase(
        &mut self,
        api: &api::Api,
        good: space_traders_client::models::TradeSymbol,
        volume: i32,
        database_pool: &sqlx::PgPool,
        reason: sql::TransactionReason,
    ) -> anyhow::Result<()> {
        let purchase_data = api
            .purchase_cargo(
                &self.symbol,
                Some(space_traders_client::models::PurchaseCargoRequest {
                    symbol: good,
                    units: volume,
                }),
            )
            .await
            .unwrap();

        self.cargo.update(&purchase_data.data.cargo);

        sql::Agent::insert(database_pool, &sql::Agent::from(*purchase_data.data.agent)).await?;

        let transaction =
            sql::MarketTransaction::try_from(purchase_data.data.transaction.as_ref().clone())?
                .with(reason);
        sql::MarketTransaction::insert(database_pool, &transaction).await?;

        Ok(())
    }

    pub async fn deliver_contract(
        &mut self,
        contract_id: &str,
        trade_symbol: space_traders_client::models::TradeSymbol,
        units: i32,
        api: &api::Api,
    ) -> anyhow::Result<space_traders_client::models::DeliverContract200Response> {
        let delivery_result: space_traders_client::models::DeliverContract200Response = api
            .deliver_contract(
                contract_id,
                Some(space_traders_client::models::DeliverContractRequest {
                    units,
                    ship_symbol: self.symbol.clone(),
                    trade_symbol: trade_symbol.to_string(),
                }),
            )
            .await?;

        self.cargo.update(&delivery_result.data.cargo);

        Ok(delivery_result)
    }

    pub async fn update_market(
        &self,
        api: &api::Api,
        database_pool: &sqlx::PgPool,
    ) -> anyhow::Result<()> {
        let market_data = api
            .get_market(&self.nav.system_symbol, &self.nav.waypoint_symbol)
            .await?;
        crate::workers::market_scrapers::update_market(*market_data.data, database_pool).await;

        Ok(())
    }
}

impl super::models::CargoState {
    pub fn get_amount(&self, symbol: &space_traders_client::models::TradeSymbol) -> i32 {
        self.inventory
            .iter()
            .find_map(|(k, v)| if k == symbol { Some(*v) } else { None })
            .unwrap_or(0)
    }

    pub fn has(&self, symbol: &space_traders_client::models::TradeSymbol) -> bool {
        self.get_amount(symbol) > 0
    }

    pub fn update(&mut self, ship_cargo: &space_traders_client::models::ShipCargo) {
        self.capacity = ship_cargo.capacity;
        self.units = ship_cargo.units;
        self.inventory = ship_cargo
            .inventory
            .iter()
            .map(|f| (f.symbol, f.units))
            .collect();
    }

    pub fn remove_cargo(
        &mut self,
        symbol: &space_traders_client::models::TradeSymbol,
        amount: i32,
    ) -> anyhow::Result<i32> {
        let amount_in = self.get_amount(symbol);
        if amount_in > amount {
            self.inventory
                .iter_mut()
                .find_map(|(k, v)| if k == symbol { Some(v) } else { None })
                .unwrap()
                .sub_assign(amount);
            Ok(self.get_amount(symbol))
        } else {
            Err(anyhow::anyhow!("Not enough cargo"))
        }
    }
}
