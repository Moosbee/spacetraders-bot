use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnectorAsync, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "DBScrapTransaction")]
pub struct ScrapTransaction {
    pub id: i64,
    pub waypoint_symbol: String,
    pub ship_symbol: String,
    pub total_price: i32,
    pub timestamp: DateTime<Utc>,
}
impl TryFrom<models::ScrapTransaction> for ScrapTransaction {
    type Error = crate::Error;

    fn try_from(item: models::ScrapTransaction) -> Result<Self, Self::Error> {
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)
            .map_err(|_| Self::Error::InvalidTimestamp(item.timestamp))?;
        Ok(Self {
            id: 0,
            waypoint_symbol: item.waypoint_symbol,
            ship_symbol: item.ship_symbol,
            total_price: item.total_price,
            timestamp,
        })
    }
}

impl ScrapTransaction {
    pub async fn get_by_ship(
        database_pool: &super::DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ScrapTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ScrapTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM scrap_transaction
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
                    ScrapTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM scrap_transaction
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
                        FROM scrap_transaction
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

    pub async fn get_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ScrapTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ScrapTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM scrap_transaction
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
                    ScrapTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM scrap_transaction
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
                        FROM scrap_transaction
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

    pub async fn get_by_system(
        database_pool: &super::DbPool,
        symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ScrapTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ScrapTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM scrap_transaction
                        JOIN waypoint ON scrap_transaction.waypoint_symbol = waypoint.symbol
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
                    ScrapTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM scrap_transaction
                        JOIN waypoint ON scrap_transaction.waypoint_symbol = waypoint.symbol
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
                        FROM scrap_transaction
                        JOIN waypoint ON scrap_transaction.waypoint_symbol = waypoint.symbol
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

impl DatabaseConnectorAsync for ScrapTransaction {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert_new(
        database_pool: &super::DbPool,
        item: &ScrapTransaction,
    ) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
                INSERT INTO scrap_transaction (
                    waypoint_symbol,
                    ship_symbol,
                    total_price,
                    "timestamp"
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (waypoint_symbol, ship_symbol, "timestamp") DO NOTHING
                RETURNING id
            "#,
            item.waypoint_symbol,
            item.ship_symbol,
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
                FROM scrap_transaction
                WHERE waypoint_symbol = $1
                  AND ship_symbol = $2
                  AND "timestamp" = $3
                LIMIT 1
            "#,
            item.waypoint_symbol,
            item.ship_symbol,
            item.timestamp
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;

        Ok(existing.id.into())
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn upsert(database_pool: &super::DbPool, item: &ScrapTransaction) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO scrap_transaction (
                    waypoint_symbol,
                    ship_symbol,
                    total_price,
                    "timestamp"
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (waypoint_symbol, ship_symbol, "timestamp") DO NOTHING
            "#,
            item.waypoint_symbol,
            item.ship_symbol,
            item.total_price,
            item.timestamp
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn update(database_pool: &super::DbPool, item: &ScrapTransaction) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE scrap_transaction
                SET
                    waypoint_symbol = $1,
                    ship_symbol = $2,
                    total_price = $3,
                    "timestamp" = $4
                WHERE id = $5
            "#,
            item.waypoint_symbol,
            item.ship_symbol,
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
        items: &[ScrapTransaction],
    ) -> crate::Result<()> {
        let (waypoint_symbols, ship_symbols, total_prices, timestamps): (
            Vec<String>,
            Vec<String>,
            Vec<i32>,
            Vec<DateTime<Utc>>,
        ) = itertools::multiunzip(items.iter().map(|t| {
            (
                t.waypoint_symbol.clone(),
                t.ship_symbol.clone(),
                t.total_price,
                t.timestamp,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO scrap_transaction (
                waypoint_symbol,
                ship_symbol,
                total_price,
                "timestamp"
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::integer[],
                $4::timestamp[]
            )
            ON CONFLICT (waypoint_symbol, ship_symbol, "timestamp") DO NOTHING
            "#,
            &waypoint_symbols,
            &ship_symbols,
            &total_prices,
            &timestamps as &[DateTime<Utc>],
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ScrapTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ScrapTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM scrap_transaction
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
                    ScrapTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM scrap_transaction
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
                        FROM scrap_transaction
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
            ScrapTransaction,
            r#"
                SELECT
                    id,
                    waypoint_symbol,
                    ship_symbol,
                    total_price,
                    "timestamp"
                FROM scrap_transaction
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
    async fn delete_by_id(database_pool: &super::DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM scrap_transaction
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
