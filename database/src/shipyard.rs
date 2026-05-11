use super::{run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult};
use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBShipyard")]
pub struct Shipyard {
    #[allow(dead_code)]
    pub id: i64,
    pub waypoint_symbol: String,
    pub modifications_fee: i32,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl From<&models::Shipyard> for Shipyard {
    fn from(item: &models::Shipyard) -> Self {
        Self {
            id: 0,
            waypoint_symbol: item.symbol.clone(),
            modifications_fee: item.modifications_fee,
            created_at: sqlx::types::chrono::DateTime::<Utc>::MIN_UTC,
        }
    }
}

impl Shipyard {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn insert_get_id(
        database_pool: &super::DbPool,
        item: &Shipyard,
    ) -> crate::Result<i64> {
        let id = sqlx::query!(
            r#"
              INSERT INTO shipyard (
                  waypoint_symbol,
                  modifications_fee
              )
              VALUES ($1, $2)
              RETURNING id
          "#,
            item.waypoint_symbol,
            item.modifications_fee
        )
        .fetch_one(&database_pool.database_pool)
        .await?
        .id;
        Ok(id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Option<Shipyard>> {
        let erg = sqlx::query_as!(
            Shipyard,
            r#"
            SELECT DISTINCT ON (waypoint_symbol)
                id,
                waypoint_symbol,
                modifications_fee,
                created_at
            FROM shipyard
            WHERE waypoint_symbol = $1
            ORDER BY waypoint_symbol, created_at DESC
            "#,
            waypoint_symbol
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last(database_pool: &super::DbPool) -> crate::Result<Vec<Shipyard>> {
        Self::get_last_paginated(database_pool, PaginatedQuery::unpaged())
            .await
            .map(|result| result.items)
    }

    pub async fn get_history_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Shipyard>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Shipyard,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            modifications_fee,
                            created_at
                        FROM shipyard
                        WHERE waypoint_symbol = $1
                        ORDER BY created_at DESC, id DESC
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
                    Shipyard,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            modifications_fee,
                            created_at
                        FROM shipyard
                        WHERE waypoint_symbol = $1
                        ORDER BY created_at DESC, id DESC
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
                        FROM shipyard
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

    pub async fn get_last_paginated(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Shipyard>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Shipyard,
                    r#"
                        SELECT DISTINCT ON (waypoint_symbol)
                            id,
                            waypoint_symbol,
                            modifications_fee,
                            created_at
                        FROM shipyard
                        ORDER BY waypoint_symbol, created_at DESC
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
                    Shipyard,
                    r#"
                        SELECT DISTINCT ON (waypoint_symbol)
                            id,
                            waypoint_symbol,
                            modifications_fee,
                            created_at
                        FROM shipyard
                        ORDER BY waypoint_symbol, created_at DESC
                    "#
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(DISTINCT waypoint_symbol) as "count!"
                        FROM shipyard
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

impl DatabaseConnectorAsync for Shipyard {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &super::DbPool, item: &Shipyard) -> crate::Result<Self::ID> {
        Self::insert_get_id(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &super::DbPool, item: &Shipyard) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO shipyard (
                    waypoint_symbol,
                    modifications_fee
                )
                VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE
                SET waypoint_symbol = EXCLUDED.waypoint_symbol,
                    modifications_fee = EXCLUDED.modifications_fee
            "#,
            item.waypoint_symbol,
            item.modifications_fee
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &super::DbPool, item: &Shipyard) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE shipyard
                SET
                    waypoint_symbol = $1,
                    modifications_fee = $2
                WHERE id = $3
            "#,
            item.waypoint_symbol,
            item.modifications_fee,
            item.id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[Shipyard]) -> crate::Result<()> {
        let (waypoint_symbols, modifications_fees): (Vec<String>, Vec<i32>) = itertools::multiunzip(
            items
                .iter()
                .map(|s| (s.waypoint_symbol.clone(), s.modifications_fee)),
        );

        sqlx::query!(
            r#"
            INSERT INTO shipyard (
                waypoint_symbol,
                modifications_fee
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::integer[]
            )
            ON CONFLICT (id) DO UPDATE
            SET waypoint_symbol = EXCLUDED.waypoint_symbol,
                modifications_fee = EXCLUDED.modifications_fee
            "#,
            &waypoint_symbols,
            &modifications_fees as &[i32],
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Shipyard>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Shipyard,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            modifications_fee,
                            created_at
                        FROM shipyard
                        ORDER BY created_at DESC, id DESC
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
                    Shipyard,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            modifications_fee,
                            created_at
                        FROM shipyard
                        ORDER BY created_at DESC, id DESC
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
                        FROM shipyard
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
        let erg = sqlx::query_as!(
            Shipyard,
            r#"
            SELECT
                id,
                waypoint_symbol,
                modifications_fee,
                created_at
            FROM shipyard
            WHERE id = $1
            "#,
            *id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(
        database_pool: &super::DbPool,
        id: &Self::ID,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM shipyard
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
