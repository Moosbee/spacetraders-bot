use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::{run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult};

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBShipyardShipTypes")]
pub struct ShipyardShipTypes {
    #[allow(dead_code)]
    pub id: i64,
    pub shipyard_id: i64,
    pub ship_type: models::ShipType,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl ShipyardShipTypes {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipyardShipTypes>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipyardShipTypes,
                    r#"
                        SELECT
                            id,
                            shipyard_id,
                            ship_type as "ship_type: models::ShipType",
                            created_at
                        FROM shipyard_ship_types
                        WHERE shipyard_id = (
                            SELECT id FROM shipyard WHERE waypoint_symbol = $1 ORDER BY created_at DESC LIMIT 1
                        )
                        ORDER BY ship_type ASC, created_at DESC
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
                    ShipyardShipTypes,
                    r#"
                        SELECT
                            id,
                            shipyard_id,
                            ship_type as "ship_type: models::ShipType",
                            created_at
                        FROM shipyard_ship_types
                        WHERE shipyard_id = (
                            SELECT id FROM shipyard WHERE waypoint_symbol = $1 ORDER BY created_at DESC LIMIT 1
                        )
                        ORDER BY ship_type ASC, created_at DESC
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
                        FROM shipyard_ship_types
                        WHERE shipyard_id = (
                            SELECT id FROM shipyard WHERE waypoint_symbol = $1 ORDER BY created_at DESC LIMIT 1
                        )
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

    pub async fn get_last_by_system(
        database_pool: &super::DbPool,
        system_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipyardShipTypes>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipyardShipTypes,
                    r#"
                        SELECT
                            id,
                            shipyard_id,
                            ship_type as "ship_type: models::ShipType",
                            shipyard_ship_types.created_at
                        FROM shipyard_ship_types
                        WHERE shipyard_id = ANY(
                            SELECT DISTINCT ON (shipyard.waypoint_symbol) shipyard.id FROM shipyard JOIN waypoint ON shipyard.waypoint_symbol = waypoint.symbol
                            WHERE waypoint.system_symbol = $1
                            ORDER BY shipyard.waypoint_symbol, shipyard.created_at DESC
                        )
                        ORDER BY shipyard_id ASC, ship_type ASC, shipyard_ship_types.created_at DESC
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
                    ShipyardShipTypes,
                    r#"
                        SELECT
                            id,
                            shipyard_id,
                            ship_type as "ship_type: models::ShipType",
                            shipyard_ship_types.created_at
                        FROM shipyard_ship_types
                        WHERE shipyard_id = ANY(
                            SELECT DISTINCT ON (shipyard.waypoint_symbol) shipyard.id FROM shipyard JOIN waypoint ON shipyard.waypoint_symbol = waypoint.symbol
                            WHERE waypoint.system_symbol = $1
                            ORDER BY shipyard.waypoint_symbol, shipyard.created_at DESC
                        )
                        ORDER BY shipyard_id ASC, ship_type ASC, shipyard_ship_types.created_at DESC
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
                        FROM shipyard_ship_types
                        WHERE shipyard_id = ANY(
                            SELECT DISTINCT ON (shipyard.waypoint_symbol) shipyard.id FROM shipyard JOIN waypoint ON shipyard.waypoint_symbol = waypoint.symbol
                            WHERE waypoint.system_symbol = $1
                            ORDER BY shipyard.waypoint_symbol, shipyard.created_at DESC
                        )
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
}

impl DatabaseConnectorAsync for ShipyardShipTypes {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &super::DbPool,
        item: &ShipyardShipTypes,
    ) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
                INSERT INTO shipyard_ship_types (
                    shipyard_id,
                    ship_type
                )
                VALUES ($1, $2)
                RETURNING id
            "#,
            item.shipyard_id,
            item.ship_type as models::ShipType
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(inserted.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &super::DbPool, item: &ShipyardShipTypes) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO shipyard_ship_types (
                    shipyard_id,
                    ship_type
                )
                VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE
                SET shipyard_id = EXCLUDED.shipyard_id,
                    ship_type = EXCLUDED.ship_type
            "#,
            item.shipyard_id,
            item.ship_type as models::ShipType
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(
        database_pool: &super::DbPool,
        item: &ShipyardShipTypes,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE shipyard_ship_types
                SET
                    shipyard_id = $1,
                    ship_type = $2
                WHERE id = $3
            "#,
            item.shipyard_id,
            item.ship_type as models::ShipType,
            item.id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[ShipyardShipTypes],
    ) -> crate::Result<()> {
        let (shipyard_ids, ship_types): (Vec<i64>, Vec<models::ShipType>) =
            itertools::multiunzip(items.iter().map(|s| (s.shipyard_id, s.ship_type)));

        sqlx::query!(
            r#"
            INSERT INTO shipyard_ship_types (
                shipyard_id,
                ship_type
            )
            SELECT * FROM UNNEST(
                $1::bigint[],
                $2::ship_type[]
            )
            ON CONFLICT (id) DO UPDATE
            SET shipyard_id = EXCLUDED.shipyard_id,
                ship_type = EXCLUDED.ship_type
            "#,
            &shipyard_ids,
            &ship_types as &[models::ShipType]
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipyardShipTypes>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipyardShipTypes,
                    r#"
                        SELECT
                            id,
                            shipyard_id,
                            ship_type as "ship_type: models::ShipType",
                            created_at
                        FROM shipyard_ship_types
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
                    ShipyardShipTypes,
                    r#"
                        SELECT
                            id,
                            shipyard_id,
                            ship_type as "ship_type: models::ShipType",
                            created_at
                        FROM shipyard_ship_types
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
                        FROM shipyard_ship_types
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
            ShipyardShipTypes,
            r#"
                SELECT
                    id,
                    shipyard_id,
                    ship_type as "ship_type: models::ShipType",
                    created_at
                FROM shipyard_ship_types
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
                DELETE FROM shipyard_ship_types
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
