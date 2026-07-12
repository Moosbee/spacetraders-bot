use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use crate::{run_paginated_query, DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult};

#[derive(
    Clone,
    Default,
    Debug,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    async_graphql::SimpleObject,
)]
#[graphql(name = "DBChartTransaction")]
pub struct ChartTransaction {
    pub id: i64,
    /// The symbol of the waypoint.
    pub waypoint_symbol: String, // only one per waypoint
    /// The symbol of the ship.
    pub ship_symbol: String,
    /// The total price of the transaction.
    pub total_price: i32,
    /// The timestamp of the transaction.
    pub timestamp: DateTime<Utc>,
}

impl TryFrom<models::ChartTransaction> for ChartTransaction {
    type Error = crate::Error;
    fn try_from(item: models::ChartTransaction) -> Result<Self, Self::Error> {
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)?;

        Ok(Self {
            id: 0,
            waypoint_symbol: item.waypoint_symbol,
            ship_symbol: item.ship_symbol,
            total_price: item.total_price,
            timestamp,
        })
    }
}

impl ChartTransaction {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship_symbol(
        database_pool: &DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ChartTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ChartTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM chart_transaction
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
                    ChartTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM chart_transaction
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
                        FROM chart_transaction
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

    pub async fn get_by_waypoint_symbol(
        database_pool: &DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Option<ChartTransaction>> {
        let erg = sqlx::query_as!(
            ChartTransaction,
            r#" 
          SELECT
            id,
            waypoint_symbol,
            ship_symbol,
            total_price,
            "timestamp"
          FROM chart_transaction
          WHERE waypoint_symbol = $1
          order by "timestamp"
          LIMIT 1
        "#,
            waypoint_symbol
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_system(
        database_pool: &DbPool,
        symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ChartTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ChartTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM chart_transaction
                        JOIN waypoint ON chart_transaction.waypoint_symbol = waypoint.symbol
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
                    ChartTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM chart_transaction
                        JOIN waypoint ON chart_transaction.waypoint_symbol = waypoint.symbol
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
                        FROM chart_transaction
                        JOIN waypoint ON chart_transaction.waypoint_symbol = waypoint.symbol
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

impl DatabaseConnectorAsync for ChartTransaction {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool, item), err(Debug))]
    async fn insert_new(database_pool: &DbPool, item: &ChartTransaction) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
              INSERT INTO chart_transaction (
                waypoint_symbol,
                ship_symbol,
                total_price,
                "timestamp"
              )
              VALUES ($1, $2, $3, $4)
              ON CONFLICT (waypoint_symbol) DO UPDATE SET
                ship_symbol = EXCLUDED.ship_symbol,
                total_price = EXCLUDED.total_price,
                "timestamp" = EXCLUDED."timestamp"
              RETURNING id
          "#,
            &item.waypoint_symbol,
            &item.ship_symbol,
            &item.total_price,
            &item.timestamp
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        Ok(inserted.id.into())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &ChartTransaction) -> crate::Result<()> {
        sqlx::query!(
            r#"
              INSERT INTO chart_transaction (
                waypoint_symbol,
                ship_symbol,
                total_price,
                "timestamp"
              )
              VALUES ($1, $2, $3, $4)
              ON CONFLICT (waypoint_symbol) DO UPDATE SET
                ship_symbol = EXCLUDED.ship_symbol,
                total_price = EXCLUDED.total_price,
                "timestamp" = EXCLUDED."timestamp";
          "#,
            &item.waypoint_symbol,
            &item.ship_symbol,
            &item.total_price,
            &item.timestamp
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, item), err(Debug))]
    async fn update(database_pool: &DbPool, item: &ChartTransaction) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE chart_transaction
                SET
                    waypoint_symbol = $1,
                    ship_symbol = $2,
                    total_price = $3,
                    "timestamp" = $4
                WHERE id = $5
            "#,
            &item.waypoint_symbol,
            &item.ship_symbol,
            item.total_price,
            item.timestamp,
            item.id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[ChartTransaction]) -> crate::Result<()> {
        let (waypoint_symbols, ship_symbols, total_prices, timestamps): (
            Vec<String>,
            Vec<String>,
            Vec<i32>,
            Vec<DateTime<Utc>>,
        ) = itertools::multiunzip(items.iter().map(|ct| {
            (
                ct.waypoint_symbol.clone(),
                ct.ship_symbol.clone(),
                ct.total_price,
                ct.timestamp,
            )
        }));

        sqlx::query!(
            r#"
          INSERT INTO chart_transaction (
              waypoint_symbol,
              ship_symbol,
              total_price,
              "timestamp"
          )
          SELECT waypoint, ship, price, ts FROM UNNEST(
              $1::character varying[],
              $2::character varying[],
              $3::integer[],
              $4::timestamp with time zone[]
          ) AS t(waypoint, ship, price, ts)
          ON CONFLICT (waypoint_symbol) DO UPDATE
          SET ship_symbol = EXCLUDED.ship_symbol,
              total_price = EXCLUDED.total_price,
              "timestamp" = EXCLUDED."timestamp";
          "#,
            &waypoint_symbols,
            &ship_symbols,
            &total_prices,
            &timestamps
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ChartTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ChartTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM chart_transaction
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
                    ChartTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_symbol,
                            total_price,
                            "timestamp"
                        FROM chart_transaction
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
                        FROM chart_transaction
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
    async fn get_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<Option<Self>> {
        let item = sqlx::query_as!(
            ChartTransaction,
            r#"
                SELECT
                    id,
                    waypoint_symbol,
                    ship_symbol,
                    total_price,
                    "timestamp"
                FROM chart_transaction
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
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM chart_transaction
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
