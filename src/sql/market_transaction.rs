use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime};
use space_traders_client::models::{self};

use super::{DatabaseConnector, DbPool};

#[derive(Clone, Default, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarketTransaction {
    /// The symbol of the waypoint.
    pub waypoint_symbol: String,
    /// The symbol of the ship that made the transaction.
    pub ship_symbol: String,
    /// The symbol of the trade good.
    pub trade_symbol: models::TradeSymbol,
    /// The type of transaction.
    pub r#type: models::market_transaction::Type,
    /// The number of units of the transaction.
    pub units: i32,
    /// The price per unit of the transaction.
    pub price_per_unit: i32,
    /// The total price of the transaction.
    pub total_price: i32,
    /// The timestamp of the transaction.
    pub timestamp: NaiveDateTime,
    /// The reason for the transaction.
    /// pub reason: TransactionReason,
    pub contract: Option<String>,
    pub trade_route: Option<i32>,
    pub mining: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum TransactionReason {
    Contract(String),
    TradeRoute(i32),
    MiningWaypoint(String),
    #[default]
    None,
}

impl MarketTransaction {
    pub fn with(self, reason: TransactionReason) -> Self {
        match reason {
            TransactionReason::Contract(contract) => MarketTransaction {
                contract: Some(contract),
                trade_route: None,
                mining: None,
                ..self
            },
            TransactionReason::None => MarketTransaction {
                contract: None,
                trade_route: None,
                mining: None,
                ..self
            },
            TransactionReason::TradeRoute(route) => MarketTransaction {
                contract: None,
                trade_route: Some(route),
                mining: None,
                ..self
            },
            TransactionReason::MiningWaypoint(waypoint) => MarketTransaction {
                contract: None,
                trade_route: None,
                mining: Some(waypoint),
                ..self
            },
        }
    }

    pub async fn get_by_reason(
        database_pool: &DbPool,
        reason: TransactionReason,
    ) -> sqlx::Result<Vec<MarketTransaction>> {
        match reason {
            TransactionReason::Contract(contract) => {
                MarketTransaction::get_by_contract(database_pool, &contract).await
            }
            TransactionReason::None => MarketTransaction::get_all(database_pool).await,
            TransactionReason::TradeRoute(route) => {
                MarketTransaction::get_by_trade_route(database_pool, route).await
            }
            TransactionReason::MiningWaypoint(waypoint) => {
                MarketTransaction::get_by_mining_waypoint(database_pool, &waypoint).await
            }
        }
    }

    async fn get_by_contract(database_pool: &DbPool, contract: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            MarketTransaction,
            r#"
      select 
        waypoint_symbol,
        ship_symbol,trade_symbol as "trade_symbol: models::TradeSymbol",
        "type" as "type: models::market_transaction::Type",
        units,
        price_per_unit,
        total_price,
        "timestamp",
        contract,
        trade_route,
        mining
      from market_transaction
      where contract = $1
    "#,
            contract
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }

    async fn get_by_trade_route(database_pool: &DbPool, route: i32) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            MarketTransaction,
            r#"
      select 
        waypoint_symbol,
        ship_symbol,trade_symbol as "trade_symbol: models::TradeSymbol",
        "type" as "type: models::market_transaction::Type",
        units,
        price_per_unit,
        total_price,
        "timestamp",
        contract,
        trade_route,
        mining
      from market_transaction
      where trade_route = $1
    "#,
            route
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }

    async fn get_by_mining_waypoint(
        database_pool: &DbPool,
        waypoint: &str,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            MarketTransaction,
            r#"
      select 
        waypoint_symbol,
        ship_symbol,trade_symbol as "trade_symbol: models::TradeSymbol",
        "type" as "type: models::market_transaction::Type",
        units,
        price_per_unit,
        total_price,
        "timestamp",
        contract,
        trade_route,
        mining
      from market_transaction
      where mining = $1
    "#,
            waypoint
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }

    pub async fn get_by_waypoint(
        database_pool: &DbPool,
        waypoint: &str,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            MarketTransaction,
            r#"
      select 
        waypoint_symbol,
        ship_symbol,trade_symbol as "trade_symbol: models::TradeSymbol",
        "type" as "type: models::market_transaction::Type",
        units,
        price_per_unit,
        total_price,
        "timestamp",
        contract,
        trade_route,
        mining
      from market_transaction
      where waypoint_symbol = $1
    "#,
            waypoint
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl From<MarketTransaction> for models::MarketTransaction {
    fn from(val: MarketTransaction) -> Self {
        models::MarketTransaction {
            ship_symbol: val.ship_symbol,
            trade_symbol: val.trade_symbol.to_string(),
            r#type: val.r#type,
            units: val.units,
            price_per_unit: val.price_per_unit,
            total_price: val.total_price,
            timestamp: val.timestamp.to_string(),
            waypoint_symbol: val.waypoint_symbol,
        }
    }
}

impl TryFrom<models::MarketTransaction> for MarketTransaction {
    type Error = crate::error::Error;
    fn try_from(value: models::MarketTransaction) -> Result<Self, Self::Error> {
        let tr_symbol = models::TradeSymbol::from_str(&value.trade_symbol)
            .map_err(|err| crate::error::Error::General(err.to_string() + "trade_symbol"))?;
        let timestamp = DateTime::<chrono::Utc>::from_str(&value.timestamp)
            .map_err(|err| crate::error::Error::General(err.to_string() + "timestamp"))?
            .naive_utc();

        Ok(MarketTransaction {
            ship_symbol: value.ship_symbol,
            trade_symbol: tr_symbol,
            r#type: value.r#type,
            units: value.units,
            price_per_unit: value.price_per_unit,
            total_price: value.total_price,
            timestamp,
            waypoint_symbol: value.waypoint_symbol,
            // reason: TransactionReason::None,
            contract: None,
            trade_route: None,
            mining: None,
        })
    }
}

impl DatabaseConnector<MarketTransaction> for MarketTransaction {
    async fn insert(database_pool: &DbPool, item: &MarketTransaction) -> sqlx::Result<()> {
        sqlx::query!(
        r#"
            INSERT INTO market_transaction (waypoint_symbol, ship_symbol, trade_symbol, "type", units, price_per_unit, total_price, "timestamp", contract, trade_route, mining)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (waypoint_symbol, ship_symbol, trade_symbol, "timestamp") DO UPDATE
            SET units = EXCLUDED.units,
            price_per_unit = EXCLUDED.price_per_unit,
            total_price = EXCLUDED.total_price
        "#,
        item.waypoint_symbol,
        item.ship_symbol,
        item.trade_symbol as models::TradeSymbol,
        item.r#type as models::market_transaction::Type,
        item.units,
        item.price_per_unit,
        item.total_price,
        item.timestamp,
        item.contract,
        item.trade_route,
        item.mining
    )
    .execute(&database_pool.database_pool)
    .await?;

        Ok(())
    }

    async fn insert_bulk(
        database_pool: &DbPool,
        items: &Vec<MarketTransaction>,
    ) -> sqlx::Result<()> {
        let (
            t_waypoint_symbol,
            t_ship_symbol,
            t_trade_symbol,
            t_type,
            t_units,
            t_price_per_unit,
            t_total_price,
            t_timestamp,
            t_contract,
            t_trade_route,
            t_mining,
        ): (
            Vec<String>,
            Vec<String>,
            Vec<models::TradeSymbol>,
            Vec<models::market_transaction::Type>,
            Vec<i32>,
            Vec<i32>,
            Vec<i32>,
            Vec<NaiveDateTime>,
            Vec<Option<String>>,
            Vec<Option<i32>>,
            Vec<Option<String>>,
        ) = itertools::multiunzip(items.iter().map(|item| {
            (
                item.waypoint_symbol.clone(),
                item.ship_symbol.clone(),
                item.trade_symbol.clone(),
                item.r#type.clone(),
                item.units,
                item.price_per_unit,
                item.total_price,
                item.timestamp.clone(),
                item.contract.clone(),
                item.trade_route.clone(),
                item.mining.clone(),
            )
        }));

        sqlx::query!(
        r#"
            INSERT INTO market_transaction (waypoint_symbol, ship_symbol,trade_symbol, "type", units, price_per_unit, total_price, "timestamp", contract, trade_route, mining)
              SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::trade_symbol[],
                $4::market_transaction_type[],
                $5::integer[],
                $6::integer[],
                $7::integer[],
                $8::timestamp[],
                $9::character varying[],
                $10::integer[],
                $11::character varying[]
            )
            ON CONFLICT (waypoint_symbol, ship_symbol, trade_symbol, "timestamp") DO UPDATE
            SET units = EXCLUDED.units,
            price_per_unit = EXCLUDED.price_per_unit,
            total_price = EXCLUDED.total_price
        "#,
        &t_waypoint_symbol,
        &t_ship_symbol,
        &t_trade_symbol as &[models::TradeSymbol],
        &t_type as &[models::market_transaction::Type],
        &t_units,
        &t_price_per_unit,
        &t_total_price,
        &t_timestamp,
        &t_contract as &[Option<String>],
        &t_trade_route as &[Option<i32>],
        &t_mining as &[Option<String>]
    )
    .execute(&database_pool.database_pool)
    .await?;

        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<MarketTransaction>> {
        sqlx::query_as!(
            MarketTransaction,
            r#"
      select 
        waypoint_symbol,
        ship_symbol,trade_symbol as "trade_symbol: models::TradeSymbol",
        "type" as "type: models::market_transaction::Type",
        units,
        price_per_unit,
        total_price,
        "timestamp",
        contract,
        trade_route,
        mining
      from market_transaction
    "#,
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

// impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for MarketTransaction {
//     fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
//         use sqlx::Row;
//         let waypoint_symbol: String = row.try_get("waypoint_symbol")?;
//         let ship_symbol: String = row.try_get("ship_symbol")?;
//         let trade_symbol: TradeSymbol = row.try_get("trade_symbol")?;
//         let r_type: models::market_transaction::Type = row.try_get("r_type")?;
//         let units: i32 = row.try_get("units")?;
//         let price_per_unit: i32 = row.try_get("price_per_unit")?;
//         let total_price: i32 = row.try_get("total_price")?;
//         let timestamp: String = row.try_get("timestamp")?;
//         let contract: Option<String> = row.try_get("contract")?;

//         let reason = match contract {
//             Some(contract) => TransactionReason::Contract(contract),
//             None => TransactionReason::None,
//         };

//         sqlx::Result::Ok(MarketTransaction {
//             waypoint_symbol,
//             ship_symbol,
//             trade_symbol,
//             r#type: r_type,
//             units,
//             price_per_unit,
//             total_price,
//             timestamp,
//             reason,
//         })
//     }
// }
