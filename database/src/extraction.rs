use chrono::{DateTime, Utc};
use space_traders_client::models::{self};
use tracing::instrument;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "DBExtraction")]
pub struct Extraction {
    #[allow(dead_code)]
    pub id: i64,
    pub ship_symbol: String,
    pub waypoint_symbol: String,
    pub ship_info_before: i64,
    pub ship_info_after: i64,
    pub siphon: bool,
    pub yield_symbol: models::TradeSymbol,
    pub yield_units: i32,
    #[graphql(name = "survey_signature")]
    pub survey: Option<String>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl Extraction {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_waypoint_symbol(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Extraction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE waypoint_symbol = $1
                        ORDER BY created_at ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    waypoint_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE waypoint_symbol = $1
                        ORDER BY created_at ASC, id ASC
                    "#,
                    waypoint_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM extraction
                        WHERE waypoint_symbol = $1
                    "#,
                    waypoint_symbol
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_system_symbol(
        database_pool: &DbPool,
        system_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Extraction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          extraction.created_at
                        FROM extraction JOIN waypoint ON extraction.waypoint_symbol = waypoint.symbol
                        WHERE waypoint.system_symbol = $1
                        ORDER BY extraction.created_at ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    system_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          extraction.created_at
                        FROM extraction JOIN waypoint ON extraction.waypoint_symbol = waypoint.symbol
                        WHERE waypoint.system_symbol = $1
                        ORDER BY extraction.created_at ASC, id ASC
                    "#,
                    system_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM extraction JOIN waypoint ON extraction.waypoint_symbol = waypoint.symbol
                        WHERE waypoint.system_symbol = $1
                    "#,
                    system_symbol
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship(
        database_pool: &DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Extraction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE ship_symbol = $1
                        ORDER BY created_at ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    ship_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE ship_symbol = $1
                        ORDER BY created_at ASC, id ASC
                    "#,
                    ship_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM extraction
                        WHERE ship_symbol = $1
                    "#,
                    ship_symbol
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_trade_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Extraction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE yield_symbol = $1
                        ORDER BY created_at ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    *trade_symbol as models::TradeSymbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE yield_symbol = $1
                        ORDER BY created_at ASC, id ASC
                    "#,
                    *trade_symbol as models::TradeSymbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM extraction
                        WHERE yield_symbol = $1
                    "#,
                    *trade_symbol as models::TradeSymbol
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_siphon(
        database_pool: &DbPool,
        siphon: bool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Extraction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE siphon = $1
                        ORDER BY created_at ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    siphon,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE siphon = $1
                        ORDER BY created_at ASC, id ASC
                    "#,
                    siphon
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM extraction
                        WHERE siphon = $1
                    "#,
                    siphon
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_survey_symbol(
        database_pool: &DbPool,
        survey_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Extraction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE survey = $1
                        ORDER BY created_at ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    survey_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        WHERE survey = $1
                        ORDER BY created_at ASC, id ASC
                    "#,
                    survey_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM extraction
                        WHERE survey = $1
                    "#,
                    survey_symbol
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
    }
}

impl DatabaseConnectorAsync for Extraction {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert_new(database_pool: &DbPool, item: &Extraction) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
                INSERT INTO extraction (
                  ship_symbol,
                  waypoint_symbol,
                  ship_info_before,
                  ship_info_after,
                  siphon,
                  yield_symbol,
                  yield_units,
                  survey
                )
                VALUES (
                  $1,
                  $2,
                  $3,
                  $4,
                  $5,
                  $6,
                  $7,
                  $8
                )
                RETURNING id
            "#,
            &item.ship_symbol,
            &item.waypoint_symbol,
            &item.ship_info_before,
            &item.ship_info_after,
            &item.siphon,
            &item.yield_symbol as &models::TradeSymbol,
            &item.yield_units,
            &item.survey as &Option<String>,
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(inserted.id)
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn upsert(database_pool: &DbPool, item: &Extraction) -> crate::Result<()> {
        if item.id == 0 {
            let _ = Self::insert_new(database_pool, item).await?;
            Ok(())
        } else {
            Self::update(database_pool, item).await
        }
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[Extraction]) -> crate::Result<()> {
        for item in items {
            Self::upsert(database_pool, item).await?;
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &Extraction) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE extraction
                SET
                    ship_symbol = $1,
                    waypoint_symbol = $2,
                    ship_info_before = $3,
                    ship_info_after = $4,
                    siphon = $5,
                    yield_symbol = $6,
                    yield_units = $7,
                    survey = $8,
                    created_at = $9
                WHERE id = $10
            "#,
            &item.ship_symbol,
            &item.waypoint_symbol,
            &item.ship_info_before,
            &item.ship_info_after,
            &item.siphon,
            &item.yield_symbol as &models::TradeSymbol,
            &item.yield_units,
            &item.survey as &Option<String>,
            &item.created_at,
            item.id,
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Extraction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        ORDER BY created_at ASC, id ASC
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
                    Extraction,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          waypoint_symbol,
                          ship_info_before,
                          ship_info_after,
                          siphon,
                          yield_symbol as "yield_symbol: models::TradeSymbol",
                          yield_units,
                          survey,
                          created_at
                        FROM extraction
                        ORDER BY created_at ASC, id ASC
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
                        FROM extraction
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
        let erg = sqlx::query_as!(
            Extraction,
            r#"
            SELECT
              id,
              ship_symbol,
              waypoint_symbol,
              ship_info_before,
              ship_info_after,
              siphon,
              yield_symbol as "yield_symbol: models::TradeSymbol",
              yield_units,
              survey,
              created_at
            FROM extraction
            WHERE id = $1
            LIMIT 1
        "#,
            *id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM extraction
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
