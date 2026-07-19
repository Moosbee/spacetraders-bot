use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBConstructionMaterial")]
pub struct ConstructionMaterial {
    pub id: i64,
    pub waypoint_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub required: i32,
    pub fulfilled: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConstructionMaterialSummary {
    pub id: i64,
    pub waypoint_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub required: i32,
    pub fulfilled: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sum: Option<i32>,
    pub expenses: Option<i32>,
    pub income: Option<i32>,
}

impl ConstructionMaterial {
    pub fn from(value: &models::ConstructionMaterial, waypoint_symbol: &str) -> Self {
        ConstructionMaterial {
            id: 0,
            waypoint_symbol: waypoint_symbol.to_string(),
            trade_symbol: value.trade_symbol,
            required: value.required,
            fulfilled: value.fulfilled,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_trade_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionMaterial>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionMaterial,
                    r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              created_at,
              updated_at
            FROM construction_material
            WHERE trade_symbol = $1
            ORDER BY waypoint_symbol ASC, id ASC
            LIMIT $2 OFFSET $3
          "#,
                    *trade_symbol as models::TradeSymbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ConstructionMaterial,
                    r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              created_at,
              updated_at
            FROM construction_material
            WHERE trade_symbol = $1
            ORDER BY waypoint_symbol ASC, id ASC
          "#,
                    *trade_symbol as models::TradeSymbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
            SELECT COUNT(*) as "count!"
            FROM construction_material
            WHERE trade_symbol = $1
          "#,
                    *trade_symbol as models::TradeSymbol
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
        database_pool: &DbPool,
        system_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionMaterial>> {
        run_paginated_query(
      query,
      |page_size, offset| async move {
        let items = sqlx::query_as!(
          ConstructionMaterial,
          r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              construction_material.created_at,
              construction_material.updated_at
            FROM construction_material JOIN waypoint ON construction_material.waypoint_symbol = waypoint.symbol
            WHERE waypoint.system_symbol = $1
            ORDER BY waypoint_symbol ASC, id ASC
            LIMIT $2 OFFSET $3
          "#,
          system_symbol,
          page_size,
          offset
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(items)
      },
      || async move {
        let items = sqlx::query_as!(
          ConstructionMaterial,
          r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              construction_material.created_at,
              construction_material.updated_at
            FROM construction_material JOIN waypoint ON construction_material.waypoint_symbol = waypoint.symbol
            WHERE waypoint.system_symbol = $1
            ORDER BY waypoint_symbol ASC, id ASC
          "#,
          system_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(items)
      },
      || async move {
        let count = sqlx::query!(
          r#"
            SELECT COUNT(*) as "count!"
            FROM construction_material JOIN waypoint ON construction_material.waypoint_symbol = waypoint.symbol
            WHERE waypoint.system_symbol = $1
          "#,
          system_symbol
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
        database_pool: &DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionMaterial>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionMaterial,
                    r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              created_at,
              updated_at
            FROM construction_material
            WHERE waypoint_symbol = $1
            ORDER BY trade_symbol ASC, id ASC
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
                    ConstructionMaterial,
                    r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              created_at,
              updated_at
            FROM construction_material
            WHERE waypoint_symbol = $1
            ORDER BY trade_symbol ASC, id ASC
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
            FROM construction_material
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
    pub async fn get_unfulfilled(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionMaterial>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionMaterial,
                    r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              created_at,
              updated_at
            FROM construction_material
            WHERE fulfilled < required
            ORDER BY id ASC
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
                    ConstructionMaterial,
                    r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              created_at,
              updated_at
            FROM construction_material
            WHERE fulfilled < required
            ORDER BY id ASC
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
            FROM construction_material
            WHERE fulfilled < required
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
    pub async fn get_summary(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionMaterialSummary>> {
        run_paginated_query(
      query,
      |page_size, offset| async move {
        let items = sqlx::query_as!(
          ConstructionMaterialSummary,
          r#"
            SELECT
            CONSTRUCTION_MATERIAL.ID,
            CONSTRUCTION_MATERIAL.WAYPOINT_SYMBOL,
            CONSTRUCTION_MATERIAL.TRADE_SYMBOL AS "trade_symbol: models::TradeSymbol",
            CONSTRUCTION_MATERIAL.REQUIRED,
            CONSTRUCTION_MATERIAL.FULFILLED,
            CONSTRUCTION_MATERIAL.CREATED_AT,
            CONSTRUCTION_MATERIAL.UPDATED_AT,
            SUM(MARKET_TRANSACTION.TOTAL_PRICE) AS "sum: i32",
            SUM(
              CASE
              WHEN MARKET_TRANSACTION.TYPE = 'PURCHASE' THEN MARKET_TRANSACTION.TOTAL_PRICE
              ELSE 0
              END
            ) AS "expenses: i32",
            SUM(
              CASE
              WHEN MARKET_TRANSACTION.TYPE = 'PURCHASE' THEN 0
              ELSE MARKET_TRANSACTION.TOTAL_PRICE
              END
            ) AS "income: i32"
            FROM
            CONSTRUCTION_MATERIAL
            LEFT JOIN PUBLIC.CONSTRUCTION_SHIPMENT ON CONSTRUCTION_SHIPMENT.MATERIAL_ID = CONSTRUCTION_MATERIAL.ID
            LEFT JOIN PUBLIC.MARKET_TRANSACTION ON CONSTRUCTION_SHIPMENT.ID = MARKET_TRANSACTION.CONSTRUCTION
            GROUP BY CONSTRUCTION_MATERIAL.ID
            ORDER BY CONSTRUCTION_MATERIAL.ID ASC
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
          ConstructionMaterialSummary,
          r#"
            SELECT
            CONSTRUCTION_MATERIAL.ID,
            CONSTRUCTION_MATERIAL.WAYPOINT_SYMBOL,
            CONSTRUCTION_MATERIAL.TRADE_SYMBOL AS "trade_symbol: models::TradeSymbol",
            CONSTRUCTION_MATERIAL.REQUIRED,
            CONSTRUCTION_MATERIAL.FULFILLED,
            CONSTRUCTION_MATERIAL.CREATED_AT,
            CONSTRUCTION_MATERIAL.UPDATED_AT,
            SUM(MARKET_TRANSACTION.TOTAL_PRICE) AS "sum: i32",
            SUM(
              CASE
              WHEN MARKET_TRANSACTION.TYPE = 'PURCHASE' THEN MARKET_TRANSACTION.TOTAL_PRICE
              ELSE 0
              END
            ) AS "expenses: i32",
            SUM(
              CASE
              WHEN MARKET_TRANSACTION.TYPE = 'PURCHASE' THEN 0
              ELSE MARKET_TRANSACTION.TOTAL_PRICE
              END
            ) AS "income: i32"
            FROM
            CONSTRUCTION_MATERIAL
            LEFT JOIN PUBLIC.CONSTRUCTION_SHIPMENT ON CONSTRUCTION_SHIPMENT.MATERIAL_ID = CONSTRUCTION_MATERIAL.ID
            LEFT JOIN PUBLIC.MARKET_TRANSACTION ON CONSTRUCTION_SHIPMENT.ID = MARKET_TRANSACTION.CONSTRUCTION
            GROUP BY CONSTRUCTION_MATERIAL.ID
            ORDER BY CONSTRUCTION_MATERIAL.ID ASC
          "#,
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(items)
      },
      || async move {
        let count = sqlx::query!(
          r#"
            SELECT COUNT(*) as "count!"
            FROM construction_material
          "#
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(count.count)
      },
    )
    .await
    }
}

impl DatabaseConnectorAsync for ConstructionMaterial {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &DbPool,
        item: &ConstructionMaterial,
    ) -> crate::Result<Self::ID> {
        let record = sqlx::query!(
            r#"
        INSERT INTO construction_material (
          waypoint_symbol,
          trade_symbol,
          required,
          fulfilled,
          updated_at
        )
        VALUES ($1, $2::trade_symbol, $3, $4, NOW())
        ON CONFLICT (waypoint_symbol, trade_symbol) DO UPDATE SET
          required = EXCLUDED.required,
          fulfilled = EXCLUDED.fulfilled,
          updated_at = NOW()
        RETURNING id;
      "#,
            &item.waypoint_symbol,
            &item.trade_symbol as &models::TradeSymbol,
            &item.required,
            &item.fulfilled
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(record.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &ConstructionMaterial) -> crate::Result<()> {
        let _ = sqlx::query!(
            r#"
        INSERT INTO construction_material (
          waypoint_symbol,
          trade_symbol,
          required,
          fulfilled,
          updated_at
        )
        VALUES ($1, $2::trade_symbol, $3, $4, NOW())
        ON CONFLICT (waypoint_symbol, trade_symbol) DO UPDATE SET
          required = EXCLUDED.required,
          fulfilled = EXCLUDED.fulfilled,
          updated_at = NOW()
        RETURNING id;
      "#,
            &item.waypoint_symbol,
            &item.trade_symbol as &models::TradeSymbol,
            &item.required,
            &item.fulfilled
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, item), err(Debug))]
    async fn update(database_pool: &DbPool, item: &ConstructionMaterial) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &DbPool,
        items: &[ConstructionMaterial],
    ) -> crate::Result<()> {
        let (waypoint_symbols, trade_symbols, requireds, fulfilleds): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|cm| {
            (
                cm.waypoint_symbol.clone(),
                cm.trade_symbol,
                cm.required,
                cm.fulfilled,
            )
        }));

        sqlx::query!(
            r#"
      INSERT INTO construction_material (
        waypoint_symbol,
        trade_symbol,
        required,
        fulfilled,
        updated_at
      )
      SELECT waypoint, trade, req, ful, NOW() FROM UNNEST(
        $1::character varying[],
        $2::trade_symbol[],
        $3::integer[],
        $4::integer[]
      ) AS t(waypoint, trade, req, ful)
      ON CONFLICT (waypoint_symbol, trade_symbol) DO UPDATE
      SET required = EXCLUDED.required,
        fulfilled = EXCLUDED.fulfilled,
        updated_at = NOW();
      "#,
            &waypoint_symbols,
            &trade_symbols as &[models::TradeSymbol],
            &requireds,
            &fulfilleds
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionMaterial>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionMaterial,
                    r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              created_at,
              updated_at
            FROM construction_material
            ORDER BY waypoint_symbol ASC, trade_symbol ASC, id ASC
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
                    ConstructionMaterial,
                    r#"
            SELECT
              id,
              waypoint_symbol,
              trade_symbol as "trade_symbol: models::TradeSymbol",
              required,
              fulfilled,
              created_at,
              updated_at
            FROM construction_material
            ORDER BY waypoint_symbol ASC, trade_symbol ASC, id ASC
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
            FROM construction_material
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
            ConstructionMaterial,
            r#"
      SELECT
        id,
        waypoint_symbol,
        trade_symbol as "trade_symbol: models::TradeSymbol",
        required,
        fulfilled,
        created_at,
        updated_at
      FROM construction_material
      WHERE id = $1
      LIMIT 1
      "#,
            *id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
        DELETE FROM construction_material
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
