use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime};
use space_traders_client::models;

use super::DatabaseConnector;

pub struct ShipModificationTransaction {
    pub waypoint_symbol: String,
    pub ship_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub total_price: i32,
    pub timestamp: NaiveDateTime,
}

impl TryFrom<models::ShipModificationTransaction> for ShipModificationTransaction {
    type Error = super::shipyard_transaction::ParseError;

    fn try_from(item: models::ShipModificationTransaction) -> Result<Self, Self::Error> {
        let trade_symbol = models::TradeSymbol::from_str(&item.trade_symbol)
            .map_err(|_| Self::Error::TradeSymbol(item.trade_symbol))?;
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)
            .map_err(|_| Self::Error::Timestamp(item.timestamp))?
            .naive_utc();
        Ok(Self {
            waypoint_symbol: item.waypoint_symbol,
            ship_symbol: item.ship_symbol,
            trade_symbol,
            total_price: item.total_price,
            timestamp,
        })
    }
}

impl DatabaseConnector<ShipModificationTransaction> for ShipModificationTransaction {
    async fn insert(
        database_pool: &super::DbPool,
        item: &ShipModificationTransaction,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO ship_modification_transaction (
                    waypoint_symbol,
                    ship_symbol,
                    trade_symbol,
                    total_price,
                    "timestamp"
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (waypoint_symbol, ship_symbol, trade_symbol, "timestamp") DO NOTHING
            "#,
            item.waypoint_symbol,
            item.ship_symbol,
            item.trade_symbol as models::TradeSymbol,
            item.total_price,
            item.timestamp
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &Vec<ShipModificationTransaction>,
    ) -> sqlx::Result<()> {
        let (waypoint_symbols, ship_symbols, trade_symbols, total_prices, timestamps): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|t| {
            (
                t.waypoint_symbol.clone(),
                t.ship_symbol.clone(),
                t.trade_symbol,
                t.total_price,
                t.timestamp,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO ship_modification_transaction (
                waypoint_symbol,
                ship_symbol,
                trade_symbol,
                total_price,
                "timestamp"
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::trade_symbol[],
                $4::integer[],
                $5::timestamp[]
            )
            ON CONFLICT (waypoint_symbol, ship_symbol, trade_symbol, "timestamp") DO NOTHING
            "#,
            &waypoint_symbols,
            &ship_symbols,
            &trade_symbols as &[models::TradeSymbol],
            &total_prices,
            &timestamps
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(
        database_pool: &super::DbPool,
    ) -> sqlx::Result<Vec<ShipModificationTransaction>> {
        sqlx::query_as!(
            ShipModificationTransaction,
            r#"
            SELECT
                waypoint_symbol,
                ship_symbol,
                trade_symbol as "trade_symbol: models::TradeSymbol",
                total_price,
                "timestamp"
            FROM ship_modification_transaction
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
