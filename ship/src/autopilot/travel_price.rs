use std::collections::HashMap;

use crate::error::Result;

#[allow(async_fn_in_trait)]
pub trait TravelPriceCalc {
    async fn get_fuel_price(&mut self, waypoint_symbol: &str) -> Result<i32>;
    async fn get_antimatter_price(&mut self, waypoint_symbol: &str) -> Result<i32>;
}

#[derive(Default, Clone, Copy)]
pub struct SimpleTravelPriceCalc {}

impl TravelPriceCalc for SimpleTravelPriceCalc {
    async fn get_fuel_price(&mut self, _waypoint_symbol: &str) -> Result<i32> {
        Ok(70)
    }

    async fn get_antimatter_price(&mut self, _waypoint_symbol: &str) -> Result<i32> {
        Ok(6000)
    }
}

pub struct TravelPriceCache {
    database_pool: database::DbPool,
    fuel_prices: HashMap<String, i32>,
    antimatter_prices: HashMap<String, i32>,
    default_fuel_price: i32,
    default_antimatter_price: i32,
}

impl TravelPriceCache {
    pub fn new(database_pool: database::DbPool) -> Self {
        Self {
            database_pool,
            fuel_prices: HashMap::new(),
            antimatter_prices: HashMap::new(),
            default_fuel_price: 70,
            default_antimatter_price: 6000,
        }
    }

    pub async fn preload_system_prices(&mut self, system_symbol: &str) -> Result<()> {
        let prices = database::MarketTradeGood::get_last_by_system(
            &self.database_pool,
            system_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?;

        for item in prices
            .items
            .into_iter()
            .filter(|tg| tg.symbol == space_traders_client::models::TradeSymbol::Fuel)
        {
            self.fuel_prices
                .insert(item.waypoint_symbol.clone(), item.purchase_price);
        }

        Ok(())
    }

    pub async fn preload_antimatter_prices(&mut self, waypoints: &[String]) -> Result<()> {
        for waypoint in waypoints {
            let trade_good = database::MarketTradeGood::get_by_last_waypoint_and_trade_symbol(
                &self.database_pool,
                waypoint,
                &space_traders_client::models::TradeSymbol::Antimatter,
            )
            .await?;

            if let Some(item) = trade_good {
                self.antimatter_prices
                    .insert(waypoint.clone(), item.purchase_price);
            }
        }
        Ok(())
    }
}

impl TravelPriceCalc for TravelPriceCache {
    async fn get_fuel_price(&mut self, waypoint_symbol: &str) -> Result<i32> {
        if let Some(price) = self.fuel_prices.get(waypoint_symbol) {
            return Ok(*price);
        }
        let trade_good = database::MarketTradeGood::get_by_last_waypoint_and_trade_symbol(
            &self.database_pool,
            waypoint_symbol,
            &space_traders_client::models::TradeSymbol::Fuel,
        )
        .await?;

        if let Some(item) = trade_good {
            self.fuel_prices
                .insert(waypoint_symbol.to_string(), item.purchase_price);
            return Ok(item.purchase_price);
        }

        Ok(self.default_fuel_price)
    }

    async fn get_antimatter_price(&mut self, waypoint_symbol: &str) -> Result<i32> {
        if let Some(price) = self.antimatter_prices.get(waypoint_symbol) {
            return Ok(*price);
        }
        let trade_good = database::MarketTradeGood::get_by_last_waypoint_and_trade_symbol(
            &self.database_pool,
            waypoint_symbol,
            &space_traders_client::models::TradeSymbol::Antimatter,
        )
        .await?;

        if let Some(item) = trade_good {
            self.antimatter_prices
                .insert(waypoint_symbol.to_string(), item.purchase_price);
            return Ok(item.purchase_price);
        }

        Ok(self.default_antimatter_price)
    }
}
