use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::{run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult};

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
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship(
        database_pool: &super::DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipModificationTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
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
                        ORDER BY "timestamp" ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    ship_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
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
                        ORDER BY "timestamp" ASC, id ASC
                    "#,
                    ship_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM ship_modification_transaction
                        WHERE ship_symbol = $1
                    "#,
                    ship_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipModificationTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
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
                        ORDER BY "timestamp" ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    waypoint_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
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
                        ORDER BY "timestamp" ASC, id ASC
                    "#,
                    waypoint_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM ship_modification_transaction
                        WHERE waypoint_symbol = $1
                    "#,
                    waypoint_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_system(
        database_pool: &super::DbPool,
        symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipModificationTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
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
                        JOIN waypoint ON ship_modification_transaction.waypoint_symbol = waypoint.symbol
                        WHERE waypoint.system_symbol = $1
                        ORDER BY "timestamp" ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
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
                        JOIN waypoint ON ship_modification_transaction.waypoint_symbol = waypoint.symbol
                        WHERE waypoint.system_symbol = $1
                        ORDER BY "timestamp" ASC, id ASC
                    "#,
                    symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM ship_modification_transaction
                        JOIN waypoint ON ship_modification_transaction.waypoint_symbol = waypoint.symbol
                        WHERE waypoint.system_symbol = $1
                    "#,
                    symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }
}

impl DatabaseConnectorAsync for ShipModificationTransaction {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert_new(
        database_pool: &super::DbPool,
        item: &ShipModificationTransaction,
    ) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
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
                RETURNING id
            "#,
            item.waypoint_symbol,
            item.ship_symbol,
            item.trade_symbol as models::TradeSymbol,
            item.total_price,
            item.timestamp
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;

        if let Some(inserted) = inserted {
            return Ok(inserted.id.into());
        }

        let existing = sqlx::query!(
            r#"
                SELECT id
                FROM ship_modification_transaction
                WHERE waypoint_symbol = $1
                  AND ship_symbol = $2
                  AND trade_symbol = $3
                  AND "timestamp" = $4
                LIMIT 1
            "#,
            item.waypoint_symbol,
            item.ship_symbol,
            item.trade_symbol as models::TradeSymbol,
            item.timestamp
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;

        Ok(existing.id.into())
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn upsert(
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

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn update(
        database_pool: &super::DbPool,
        item: &ShipModificationTransaction,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE ship_modification_transaction
                SET
                    waypoint_symbol = $1,
                    ship_symbol = $2,
                    trade_symbol = $3,
                    total_price = $4,
                    "timestamp" = $5
                WHERE id = $6
            "#,
            item.waypoint_symbol,
            item.ship_symbol,
            item.trade_symbol as models::TradeSymbol,
            item.total_price,
            item.timestamp,
            item.id
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

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipModificationTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
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
                        ORDER BY "timestamp" ASC, id ASC
                        LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
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
                        ORDER BY "timestamp" ASC, id ASC
                    "#
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM ship_modification_transaction
                    "#
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_by_id(
        database_pool: &super::DbPool,
        id: &Self::ID,
    ) -> crate::Result<Option<Self>> {
        let item = sqlx::query_as!(
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
                WHERE id = $1
                LIMIT 1
            "#,
            *id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(item)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(
        database_pool: &super::DbPool,
        id: &Self::ID,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM ship_modification_transaction
                WHERE id = $1
            "#,
            *id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = id;
    }
}
