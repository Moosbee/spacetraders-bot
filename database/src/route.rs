use chrono::{DateTime, Utc};
use tracing::instrument;

use super::{DatabaseConnectorAsync, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Debug, Clone, PartialEq, async_graphql::SimpleObject)]
#[graphql(name = "DBRoute")]
pub struct Route {
    pub id: i32,
    pub ship_symbol: String,
    pub from: String,
    pub to: String,
    pub nav_mode: String,
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: f64,
    pub ship_info_before: Option<i64>,
    pub ship_info_after: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl Route {
    pub async fn get_by_ship(
        database_pool: &super::DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Route>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Route,
                    r#"
                        SELECT 
                          id,
                          ship_symbol,
                          "from",
                          "to",
                          nav_mode,
                          distance,
                          fuel_cost,
                          travel_time,
                          ship_info_before,
                          ship_info_after,
                          created_at
                        FROM route
                        WHERE ship_symbol = $1
                        ORDER BY created_at ASC, id ASC
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
                    Route,
                    r#"
                        SELECT 
                          id,
                          ship_symbol,
                          "from",
                          "to",
                          nav_mode,
                          distance,
                          fuel_cost,
                          travel_time,
                          ship_info_before,
                          ship_info_after,
                          created_at
                        FROM route
                        WHERE ship_symbol = $1
                        ORDER BY created_at ASC, id ASC
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
                        FROM route
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
}

impl DatabaseConnectorAsync for Route {
    type ID = i32;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &super::DbPool, item: &Route) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
                INSERT INTO route (
                    ship_symbol,
                    "from",
                    "to",
                    distance,
                    nav_mode,
                    fuel_cost,
                    travel_time,
                    ship_info_before,
                    ship_info_after
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id
            "#,
            item.ship_symbol,
            item.from,
            item.to,
            item.distance,
            item.nav_mode,
            item.fuel_cost,
            item.travel_time,
            item.ship_info_before,
            item.ship_info_after
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        Ok(inserted.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &super::DbPool, item: &Route) -> crate::Result<()> {
        sqlx::query!(
            r#"
            insert into route (
            ship_symbol,
            "from",
            "to",
            distance,
            nav_mode,
            fuel_cost,
            travel_time,
            ship_info_before,
            ship_info_after
            )
            values (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9
            )
            on conflict (id) do nothing
            "#,
            item.ship_symbol,
            item.from,
            item.to,
            item.distance,
            item.nav_mode,
            item.fuel_cost,
            item.travel_time,
            item.ship_info_before,
            item.ship_info_after
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &super::DbPool, item: &Route) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE route
                SET
                    ship_symbol = $1,
                    "from" = $2,
                    "to" = $3,
                    distance = $4,
                    nav_mode = $5,
                    fuel_cost = $6,
                    travel_time = $7,
                    ship_info_before = $8,
                    ship_info_after = $9
                WHERE id = $10
            "#,
            item.ship_symbol,
            item.from,
            item.to,
            item.distance,
            item.nav_mode,
            item.fuel_cost,
            item.travel_time,
            item.ship_info_before,
            item.ship_info_after,
            item.id
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[Route]) -> crate::Result<()> {
        for item in items {
            Self::upsert(database_pool, item).await?;
        }

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Route>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Route,
                    r#"
                        SELECT 
                          id,
                          ship_symbol,
                          "from",
                          "to",
                          nav_mode,
                          distance,
                          fuel_cost,
                          travel_time,
                          ship_info_before,
                          ship_info_after,
                          created_at
                        FROM route
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
                    Route,
                    r#"
                        SELECT 
                          id,
                          ship_symbol,
                          "from",
                          "to",
                          nav_mode,
                          distance,
                          fuel_cost,
                          travel_time,
                          ship_info_before,
                          ship_info_after,
                          created_at
                        FROM route
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
                        FROM route
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
            Route,
            r#"
                SELECT 
                  id,
                  ship_symbol,
                  "from",
                  "to",
                  nav_mode,
                  distance,
                  fuel_cost,
                  travel_time,
                  ship_info_before,
                  ship_info_after,
                  created_at
                FROM route
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
                DELETE FROM route
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
