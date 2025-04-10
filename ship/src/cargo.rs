use std::ops::{AddAssign, SubAssign};

use database::DatabaseConnector;
use log::debug;
use space_traders_client::models::JettisonRequest;

use crate::error;

use super::ship_models::MyShip;

enum Mode {
    Sell,
    Purchase,
}

impl MyShip {
    pub async fn purchase_cargo(
        &mut self,
        api: &space_traders_client::Api,
        symbol: &space_traders_client::models::TradeSymbol,
        units: i32,
        database_pool: &database::DbPool,
        reason: database::TransactionReason,
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
        api: &space_traders_client::Api,
        symbol: &space_traders_client::models::TradeSymbol,
        units: i32,
        database_pool: &database::DbPool,
        reason: database::TransactionReason,
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
        api: &space_traders_client::Api,
        database_pool: &database::DbPool,
    ) -> error::Result<Vec<database::MarketTradeGood>> {
        let market_info = database::MarketTradeGood::get_last_by_waypoint(
            database_pool,
            &self.nav.waypoint_symbol,
        )
        .await?;
        let market_info = if market_info.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            self.update_market(api, database_pool).await?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            database::MarketTradeGood::get_last_by_waypoint(
                database_pool,
                &self.nav.waypoint_symbol,
            )
            .await
        } else {
            sqlx::Result::Ok(market_info)
        };

        Ok(market_info?)
    }

    fn calculate_volumes(
        &self,
        quantity: i32,
        market_info: &[database::MarketTradeGood],
        good: &space_traders_client::models::TradeSymbol,
    ) -> error::Result<Vec<i32>> {
        let max_purchase_volume = market_info
            .iter()
            .find(|m| m.symbol == *good)
            .map(|m| m.trade_volume)
            .ok_or_else(|| {
                error::Error::General("Could not find good in market info".to_string())
            })?;

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
        api: &space_traders_client::Api,
        good: &space_traders_client::models::TradeSymbol,
        volume: i32,
        r_type: Mode,
        database_pool: &database::DbPool,
        reason: database::TransactionReason,
    ) -> error::Result<()> {
        self.mutate();
        let trade_data = match r_type {
            Mode::Sell => {
                let sell_data: space_traders_client::models::SellCargo201Response = api
                    .sell_cargo(
                        &self.symbol,
                        Some(space_traders_client::models::SellCargoRequest {
                            symbol: *good,
                            units: volume,
                        }),
                    )
                    .await?;

                sell_data.data
            }
            Mode::Purchase => {
                let purchase_data: space_traders_client::models::PurchaseCargo201Response = api
                    .purchase_cargo(
                        &self.symbol,
                        Some(space_traders_client::models::PurchaseCargoRequest {
                            symbol: *good,
                            units: volume,
                        }),
                    )
                    .await?;

                purchase_data.data
            }
        };

        self.cargo.update(&trade_data.cargo);
        self.notify().await;

        database::Agent::insert(database_pool, &database::Agent::from(*trade_data.agent)).await?;

        let transaction =
            database::MarketTransaction::try_from(trade_data.transaction.as_ref().clone())?
                .with(reason);
        database::MarketTransaction::insert(database_pool, &transaction).await?;

        Ok(())
    }

    pub async fn deliver_contract(
        &mut self,
        contract_id: &str,
        trade_symbol: space_traders_client::models::TradeSymbol,
        units: i32,
        api: &space_traders_client::Api,
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

    pub async fn supply_construction(
        &mut self,
        trade_symbol: space_traders_client::models::TradeSymbol,
        units: i32,
        api: &space_traders_client::Api,
    ) -> Result<space_traders_client::models::SupplyConstruction201Response, error::Error> {
        self.mutate();
        let delivery_result: space_traders_client::models::SupplyConstruction201Response = api
            .supply_construction(
                &self.nav.system_symbol,
                &self.nav.waypoint_symbol,
                Some(space_traders_client::models::SupplyConstructionRequest {
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

    pub async fn simple_transfer_cargo(
        &mut self,
        trade_symbol: space_traders_client::models::TradeSymbol,
        units: i32,
        api: &space_traders_client::Api,
        target_ship: &str,
    ) -> crate::error::Result<space_traders_client::models::TransferCargo200Response> {
        self.mutate();
        let old_units = self.cargo.get_amount(&trade_symbol);
        if old_units < units {
            return Err("Not enough cargo to transfer".into());
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

        Ok(transfer_result)
    }

    pub async fn transfer_cargo(
        &mut self,
        trade_symbol: space_traders_client::models::TradeSymbol,
        units: i32,
        api: &space_traders_client::Api,
        target_ship: &str,
    ) -> crate::error::Result<space_traders_client::models::TransferCargo200Response> {
        self.mutate();

        let transfer_result = self
            .simple_transfer_cargo(trade_symbol, units, api, target_ship)
            .await?;

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
        self.broadcaster
            .sender
            .send(update_event)
            .map_err(|err| crate::error::Error::General(err.to_string()))?;

        Ok(transfer_result)
    }

    pub async fn jettison(
        &mut self,
        api: &space_traders_client::Api,
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
        api: &space_traders_client::Api,
        database_pool: &database::DbPool,
    ) -> error::Result<()> {
        let market_data = api
            .get_market(&self.nav.system_symbol, &self.nav.waypoint_symbol)
            .await?;
        crate::utils::update_market(*market_data.data, database_pool).await?;

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
    ) -> error::Result<i32> {
        let amount_in = self.get_amount(symbol);
        if amount_in >= amount {
            self.inventory
                .iter_mut()
                .find_map(|(k, v)| if k == symbol { Some(v) } else { None })
                .unwrap()
                .sub_assign(amount);
            if self.get_amount(symbol) == 0 {
                self.inventory.remove(symbol);
            }
            Ok(self.get_amount(symbol))
        } else {
            Err(error::Error::General(format!(
                "Not enough cargo of trade symbol {} to remove {} only has {} cargo is: {:?}",
                symbol, amount, amount_in, self
            )))
        }
    }

    pub fn handle_cago_update(
        &mut self,
        units: i32,
        trade_symbol: space_traders_client::models::TradeSymbol,
    ) -> Result<(), crate::error::Error> {
        debug!("Handling cargo update: {:?} {:?}", units, trade_symbol);
        let current_count = self.inventory.iter().map(|f| f.1).sum::<i32>();
        if (current_count + units) <= self.units {
            return Err("Not enough cargo".into());
        };

        let entry = self.inventory.entry(trade_symbol);

        let count = entry.or_insert(0);

        count.add_assign(units);

        let new_count = self.inventory.iter().map(|f| f.1).sum::<i32>();

        self.units = new_count;

        Ok(())
    }
}
