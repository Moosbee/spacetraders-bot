use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::DatabaseConnector;

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "DBShipModificationTransaction")]
pub struct ShipModificationTransaction {
    pub id: i64,
    pub waypoint_symbol: String,
    pub ship_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub total_price: i32,
    pub timestamp: DateTime<Utc>,
}

impl TryFrom<models::ShipModificationTransaction> for ShipModificationTransaction {
    type Error = crate::Error;

    fn try_from(item: models::ShipModificationTransaction) -> Result<Self, Self::Error> {
        let trade_symbol = models::TradeSymbol::from_str(&item.trade_symbol)
            .map_err(|_| Self::Error::InvalidTradeSymbol(item.trade_symbol))?;
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)
            .map_err(|_| Self::Error::InvalidTimestamp(item.timestamp))?;
        Ok(Self {
            id: 0,
            waypoint_symbol: item.waypoint_symbol,
            ship_symbol: item.ship_symbol,
            trade_symbol,
            total_price: item.total_price,
            timestamp,
        })
    }
}

impl ShipModificationTransaction {
    pub async fn get_by_ship(
        database_pool: &super::DbPool,
        ship_symbol: &str,
    ) -> crate::Result<Vec<ShipModificationTransaction>> {
        let reg = sqlx::query_as!(
            ShipModificationTransaction,
            r#"
      SELECT
        id,
        waypoint_symbol,
        ship_symbol,
        trade_symbol as "trade_symbol: models::TradeSymbol",
        total_price,
        "timestamp"
      FROM ship_modification_transaction
      WHERE ship_symbol = $1
      "#,
            ship_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(reg)
    }

    pub async fn get_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<ShipModificationTransaction>> {
        let reg = sqlx::query_as!(
            ShipModificationTransaction,
            r#"
      SELECT
        id,
        waypoint_symbol,
        ship_symbol,
        trade_symbol as "trade_symbol: models::TradeSymbol",
        total_price,
        "timestamp"
      FROM ship_modification_transaction
      WHERE waypoint_symbol = $1
      "#,
            waypoint_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(reg)
    }

    pub async fn get_by_system(
        database_pool: &super::DbPool,
        symbol: &str,
    ) -> crate::Result<Vec<ShipModificationTransaction>> {
        let reg = sqlx::query_as!(
            ShipModificationTransaction,
            r#"
      SELECT
        id,
        waypoint_symbol,
        ship_symbol,
        trade_symbol as "trade_symbol: models::TradeSymbol",
        total_price,
        "timestamp"
      FROM ship_modification_transaction JOIN waypoint ON ship_modification_transaction.waypoint_symbol = waypoint.symbol
      WHERE waypoint.system_symbol = $1
      "#,
            symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(reg)
    }
}

impl DatabaseConnector<ShipModificationTransaction> for ShipModificationTransaction {
    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert(
        database_pool: &super::DbPool,
        item: &ShipModificationTransaction,
    ) -> crate::Result<()> {
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

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[ShipModificationTransaction],
    ) -> crate::Result<()> {
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
            &timestamps as &[chrono::DateTime<chrono::Utc>],
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(
        database_pool: &super::DbPool,
    ) -> crate::Result<Vec<ShipModificationTransaction>> {
        let erg = sqlx::query_as!(
            ShipModificationTransaction,
            r#"
            SELECT
                id,
                waypoint_symbol,
                ship_symbol,
                trade_symbol as "trade_symbol: models::TradeSymbol",
                total_price,
                "timestamp"
            FROM ship_modification_transaction
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
