use tracing::instrument;

use crate::{run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult};

#[derive(Debug, Clone, PartialEq, Eq, async_graphql::SimpleObject)]
#[graphql(name = "DBShipJump")]
pub struct ShipJump {
    pub id: i64,
    pub ship_symbol: String,
    pub from: String,
    pub to: String,
    pub distance: i64,
    pub ship_before: i64,
    pub ship_after: i64,
}

impl ShipJump {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship(
        database_pool: &super::DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipJump>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipJump,
                    r#"
                        SELECT
                            id,
                            ship_symbol,
                            "from",
                            "to",
                            distance,
                            ship_before,
                            ship_after
                        FROM ship_jumps
                        WHERE ship_symbol = $1
                        ORDER BY id ASC
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
                    ShipJump,
                    r#"
                        SELECT
                            id,
                            ship_symbol,
                            "from",
                            "to",
                            distance,
                            ship_before,
                            ship_after
                        FROM ship_jumps
                        WHERE ship_symbol = $1
                        ORDER BY id ASC
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
                        FROM ship_jumps
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

impl DatabaseConnectorAsync for ShipJump {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &super::DbPool,
        item: &ShipJump,
    ) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
                INSERT INTO ship_jumps (
                    ship_symbol,
                    "from",
                    "to",
                    distance,
                    ship_before,
                    ship_after
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id
            "#,
            item.ship_symbol,
            item.from,
            item.to,
            item.distance,
            item.ship_before,
            item.ship_after
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(inserted.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &super::DbPool, item: &ShipJump) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO ship_jumps (
                    ship_symbol,
                    "from",
                    "to",
                    distance,
                    ship_before,
                    ship_after
                )
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            item.ship_symbol,
            item.from,
            item.to,
            item.distance,
            item.ship_before,
            item.ship_after
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &super::DbPool, item: &ShipJump) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE ship_jumps
                SET
                    ship_symbol = $1,
                    "from" = $2,
                    "to" = $3,
                    distance = $4,
                    ship_before = $5,
                    ship_after = $6
                WHERE id = $7
            "#,
            item.ship_symbol,
            item.from,
            item.to,
            item.distance,
            item.ship_before,
            item.ship_after,
            item.id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[ShipJump]) -> crate::Result<()> {
        let (ship_symbols, froms, tos, distances, ship_befores, ship_afters): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|j| {
            (
                j.ship_symbol.clone(),
                j.from.clone(),
                j.to.clone(),
                j.distance,
                j.ship_before,
                j.ship_after,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO ship_jumps (
                ship_symbol,
                "from",
                "to",
                distance,
                ship_before,
                ship_after
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::character varying[],
                $4::bigint[],
                $5::bigint[],
                $6::bigint[]
            )
            "#,
            &ship_symbols,
            &froms,
            &tos,
            &distances,
            &ship_befores,
            &ship_afters
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipJump>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipJump,
                    r#"
                        SELECT
                            id,
                            ship_symbol,
                            "from",
                            "to",
                            distance,
                            ship_before,
                            ship_after
                        FROM ship_jumps
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
                    ShipJump,
                    r#"
                        SELECT
                            id,
                            ship_symbol,
                            "from",
                            "to",
                            distance,
                            ship_before,
                            ship_after
                        FROM ship_jumps
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
                        FROM ship_jumps
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
            ShipJump,
            r#"
                SELECT
                    id,
                    ship_symbol,
                    "from",
                    "to",
                    distance,
                    ship_before,
                    ship_after
                FROM ship_jumps
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
                DELETE FROM ship_jumps
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
