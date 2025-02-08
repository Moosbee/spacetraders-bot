use std::{
    i32,
    ops::{AddAssign, SubAssign},
};

use log::debug;
use space_traders_client::models::JettisonRequest;

use crate::{
    api, error,
    sql::{self, DatabaseConnector},
};

use super::ship_models::MyShip;

enum Mode {
    Sell,
    Purchase,
}

impl MyShip {
    pub async fn purchase_cargo(
        &mut self,
        api: &api::Api,
        symbol: &space_traders_client::models::TradeSymbol,
        units: i32,
        database_pool: &sql::DbPool,
        reason: sql::TransactionReason,
    ) -> error::Result<()> {
        self.mutate();
        let market_info = self.get_market_info(api, database_pool).await?;
        let purchase_volumes = self.calculate_volumes(units, &market_info, symbol)?;

        for volume in purchase_volumes {
            self.execute_trade(
                api,
                symbol,
                volume,
                Mode::Purchase,
                database_pool,
                reason.clone(),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn sell_cargo(
        &mut self,
        api: &api::Api,
        symbol: &space_traders_client::models::TradeSymbol,
        units: i32,
        database_pool: &sql::DbPool,
        reason: sql::TransactionReason,
    ) -> error::Result<()> {
        self.mutate();
        let market_info = self.get_market_info(api, database_pool).await?;
        let sell_volumes = self.calculate_volumes(units, &market_info, symbol)?;

        for volume in sell_volumes {
            self.execute_trade(
                api,
                symbol,
                volume,
                Mode::Sell,
                database_pool,
                reason.clone(),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn get_market_info(
        &self,
        api: &api::Api,
        database_pool: &sql::DbPool,
    ) -> error::Result<Vec<sql::MarketTradeGood>> {
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

    fn calculate_volumes(
        &self,
        quantity: i32,
        market_info: &[sql::MarketTradeGood],
        good: &space_traders_client::models::TradeSymbol,
    ) -> error::Result<Vec<i32>> {
        let max_purchase_volume = market_info
            .iter()
            .find(|m| m.symbol == *good)
            .map(|m| m.trade_volume)
            .ok_or_else(|| error::Error::General(format!("Could not find good in market info")))?;

        let mut volumes = Vec::new();
        let mut remaining = quantity;

        while remaining > 0 && max_purchase_volume > 0 {
            let volume = std::cmp::min(remaining, max_purchase_volume);
            volumes.push(volume);
            remaining -= volume;
        }

        Ok(volumes)
    }

    async fn execute_trade(
        &mut self,
        api: &api::Api,
        good: &space_traders_client::models::TradeSymbol,
        volume: i32,
        r_type: Mode,
        database_pool: &sql::DbPool,
        reason: sql::TransactionReason,
    ) -> error::Result<()> {
        self.mutate();
        let trade_data = match r_type {
            Mode::Sell => {
                let sell_data: space_traders_client::models::SellCargo201Response = api
                    .sell_cargo(
                        &self.symbol,
                        Some(space_traders_client::models::SellCargoRequest {
                            symbol: good.clone(),
                            units: volume,
                        }),
                    )
                    .await
                    .unwrap();

                sell_data.data
            }
            Mode::Purchase => {
                let purchase_data: space_traders_client::models::PurchaseCargo201Response = api
                    .purchase_cargo(
                        &self.symbol,
                        Some(space_traders_client::models::PurchaseCargoRequest {
                            symbol: good.clone(),
                            units: volume,
                        }),
                    )
                    .await
                    .unwrap();

                purchase_data.data
            }
        };

        self.cargo.update(&trade_data.cargo);
        self.notify().await;

        sql::Agent::insert(database_pool, &sql::Agent::from(*trade_data.agent)).await?;

        let transaction =
            sql::MarketTransaction::try_from(trade_data.transaction.as_ref().clone())?.with(reason);
        sql::MarketTransaction::insert(database_pool, &transaction).await?;

        Ok(())
    }

    pub async fn deliver_contract(
        &mut self,
        contract_id: &str,
        trade_symbol: space_traders_client::models::TradeSymbol,
        units: i32,
        api: &api::Api,
    ) -> Result<space_traders_client::models::DeliverContract200Response, error::Error> {
        self.mutate();
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
        self.notify().await;

        Ok(delivery_result)
    }
    pub async fn transfer_cargo(
        &mut self,
        trade_symbol: space_traders_client::models::TradeSymbol,
        units: i32,
        api: &api::Api,
        target_ship: &str,
    ) -> anyhow::Result<space_traders_client::models::TransferCargo200Response> {
        self.mutate();
        let old_units = self.cargo.get_amount(&trade_symbol);
        if old_units < units {
            return Err(anyhow::anyhow!("Not enough cargo"));
        }
        let transfer_result: space_traders_client::models::TransferCargo200Response = api
            .transfer_cargo(
                &self.symbol,
                Some(space_traders_client::models::TransferCargoRequest {
                    units,
                    ship_symbol: target_ship.to_string(),
                    trade_symbol,
                }),
            )
            .await?;

        self.cargo.update(&transfer_result.data.cargo);
        self.notify().await;
        let update_event = super::ship_models::my_ship_update::MyShipUpdate {
            symbol: target_ship.to_string(),
            update: super::ship_models::my_ship_update::ShipUpdate::CargoChange(
                super::ship_models::my_ship_update::CargoChange {
                    trade_symbol,
                    units,
                },
            ),
        };
        debug!("Sending update event: {:#?}", update_event);
        self.broadcaster.sender.send(update_event)?;

        Ok(transfer_result)
    }

    pub async fn jettison(
        &mut self,
        api: &api::Api,
        trade_symbol: space_traders_client::models::TradeSymbol,
        units: i32,
    ) -> error::Result<()> {
        self.mutate();
        let jettison_data: space_traders_client::models::Jettison200Response = api
            .jettison(
                &self.symbol,
                Some(JettisonRequest {
                    units,
                    symbol: trade_symbol,
                }),
            )
            .await?;
        self.cargo.update(&jettison_data.data.cargo);
        self.notify().await;

        Ok(())
    }

    pub async fn update_market(
        &self,
        api: &api::Api,
        database_pool: &sql::DbPool,
    ) -> error::Result<()> {
        let market_data = api
            .get_market(&self.nav.system_symbol, &self.nav.waypoint_symbol)
            .await?;
        crate::workers::market_scrapers::update_market(*market_data.data, database_pool).await;

        Ok(())
    }
}

impl super::ship_models::CargoState {
    pub fn get_amount(&self, symbol: &space_traders_client::models::TradeSymbol) -> i32 {
        self.inventory
            .iter()
            .find_map(|(k, v)| if k == symbol { Some(*v) } else { None })
            .unwrap_or(0)
    }

    pub fn get_units_no_fuel(&self) -> i32 {
        self.inventory
            .iter()
            .filter(|(k, _)| **k != space_traders_client::models::TradeSymbol::Fuel)
            .fold(0, |acc, (_, v)| acc + v)
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

    pub fn handle_cago_update(
        &mut self,
        cargo_change: super::ship_models::my_ship_update::CargoChange,
    ) -> Result<(), anyhow::Error> {
        debug!("Handling cargo update: {:?}", cargo_change);
        let current_count = self.inventory.iter().map(|f| f.1).sum::<i32>();
        if !(current_count == self.units) {
            return Err(anyhow::anyhow!("Not enough cargo"));
        };

        let entry = self.inventory.entry(cargo_change.trade_symbol);

        let count = entry.or_insert(0);

        count.add_assign(cargo_change.units);

        let new_count = self.inventory.iter().map(|f| f.1).sum::<i32>();

        self.units = new_count;

        Ok(())
    }
}
