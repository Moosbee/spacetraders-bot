use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::{run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult};

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBShipyardTransaction")]
pub struct ShipyardTransaction {
    pub id: i64,
    pub waypoint_symbol: String,
    pub ship_type: models::ShipType,
    pub price: i32,
    pub agent_symbol: String,
    pub timestamp: DateTime<Utc>,
}

impl TryFrom<models::ShipyardTransaction> for ShipyardTransaction {
    type Error = crate::Error;

    fn try_from(item: models::ShipyardTransaction) -> Result<Self, Self::Error> {
        let ship_type = models::ShipType::from_str(&item.ship_type)
            .map_err(|_| Self::Error::InvalidShipType(item.ship_type))?;
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)
            .map_err(|_| Self::Error::InvalidTimestamp(item.timestamp))?;

        Ok(Self {
            id: 0,
            waypoint_symbol: item.waypoint_symbol,
            ship_type,
            price: item.price,
            agent_symbol: item.agent_symbol,
            timestamp,
        })
    }
}

impl ShipyardTransaction {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipyardTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
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
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
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
                        FROM shipyard_transaction
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
        system: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Self>> {
        let system_qr = format!("{}-%", system);
        let page_system_qr = system_qr.clone();
        let all_system_qr = system_qr.clone();
        let count_system_qr = system_qr.clone();

        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
                        WHERE waypoint_symbol LIKE $1
                        ORDER BY "timestamp" ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    page_system_qr,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
                        WHERE waypoint_symbol LIKE $1
                        ORDER BY "timestamp" ASC, id ASC
                    "#,
                    all_system_qr
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM shipyard_transaction
                        WHERE waypoint_symbol LIKE $1
                    "#,
                    count_system_qr
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship_type(
        database_pool: &super::DbPool,
        ship_type: models::ShipType,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipyardTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
                        WHERE ship_type = $1
                        ORDER BY "timestamp" ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    ship_type as models::ShipType,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
                        WHERE ship_type = $1
                        ORDER BY "timestamp" ASC, id ASC
                    "#,
                    ship_type as models::ShipType
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM shipyard_transaction
                        WHERE ship_type = $1
                    "#,
                    ship_type as models::ShipType
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_agent(
        database_pool: &super::DbPool,
        agent_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipyardTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
                        WHERE agent_symbol = $1
                        ORDER BY "timestamp" ASC, id ASC
                        LIMIT $2 OFFSET $3
                    "#,
                    agent_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
                        WHERE agent_symbol = $1
                        ORDER BY "timestamp" ASC, id ASC
                    "#,
                    agent_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM shipyard_transaction
                        WHERE agent_symbol = $1
                    "#,
                    agent_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    pub async fn get_by_id(
        database_pool: &super::DbPool,
        id: i64,
    ) -> crate::Result<ShipyardTransaction> {
        let erg = sqlx::query_as!(
            ShipyardTransaction,
            r#"
            SELECT
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
            FROM shipyard_transaction
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    pub async fn get_by_waypoint_and_ship_type(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
        ship_type: models::ShipType,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipyardTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
                        WHERE waypoint_symbol = $1 AND ship_type = $2
                        ORDER BY "timestamp" ASC, id ASC
                        LIMIT $3 OFFSET $4
                    "#,
                    waypoint_symbol,
                    ship_type as models::ShipType,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
                        WHERE waypoint_symbol = $1 AND ship_type = $2
                        ORDER BY "timestamp" ASC, id ASC
                    "#,
                    waypoint_symbol,
                    ship_type as models::ShipType
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM shipyard_transaction
                        WHERE waypoint_symbol = $1 AND ship_type = $2
                    "#,
                    waypoint_symbol,
                    ship_type as models::ShipType
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    pub async fn insert_new(
        database_pool: &super::DbPool,
        item: &ShipyardTransaction,
    ) -> crate::Result<i64> {
        let erg = sqlx::query!(
            r#"
                INSERT INTO shipyard_transaction (
                    waypoint_symbol,
                    ship_type,
                    price,
                    agent_symbol,
                    "timestamp"
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (waypoint_symbol, ship_type, agent_symbol, "timestamp") DO NOTHING
                RETURNING id
            "#,
            item.waypoint_symbol,
            item.ship_type as models::ShipType,
            item.price,
            item.agent_symbol,
            item.timestamp
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(erg.id)
    }
}

impl DatabaseConnectorAsync for ShipyardTransaction {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &super::DbPool,
        item: &ShipyardTransaction,
    ) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
                INSERT INTO shipyard_transaction (
                    waypoint_symbol,
                    ship_type,
                    price,
                    agent_symbol,
                    "timestamp"
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (waypoint_symbol, ship_type, agent_symbol, "timestamp") DO NOTHING
                RETURNING id
            "#,
            item.waypoint_symbol,
            item.ship_type as models::ShipType,
            item.price,
            item.agent_symbol,
            item.timestamp
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;

        if let Some(inserted) = inserted {
            return Ok(inserted.id);
        }

        let existing = sqlx::query!(
            r#"
                SELECT id
                FROM shipyard_transaction
                WHERE waypoint_symbol = $1
                  AND ship_type = $2
                  AND agent_symbol = $3
                  AND "timestamp" = $4
                LIMIT 1
            "#,
            item.waypoint_symbol,
            item.ship_type as models::ShipType,
            item.agent_symbol,
            item.timestamp
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;

        Ok(existing.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(
        database_pool: &super::DbPool,
        item: &ShipyardTransaction,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO shipyard_transaction (
                    waypoint_symbol,
                    ship_type,
                    price,
                    agent_symbol,
                    "timestamp"
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (waypoint_symbol, ship_type, agent_symbol, "timestamp") DO NOTHING
            "#,
            item.waypoint_symbol,
            item.ship_type as models::ShipType,
            item.price,
            item.agent_symbol,
            item.timestamp
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(
        database_pool: &super::DbPool,
        item: &ShipyardTransaction,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE shipyard_transaction
                SET
                    waypoint_symbol = $1,
                    ship_type = $2,
                    price = $3,
                    agent_symbol = $4,
                    "timestamp" = $5
                WHERE id = $6
            "#,
            item.waypoint_symbol,
            item.ship_type as models::ShipType,
            item.price,
            item.agent_symbol,
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
        items: &[ShipyardTransaction],
    ) -> crate::Result<()> {
        let (waypoint_symbols, ship_types, prices, agent_symbols, timestamps): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|t| {
            (
                t.waypoint_symbol.clone(),
                t.ship_type,
                t.price,
                t.agent_symbol.clone(),
                t.timestamp,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO shipyard_transaction (
                waypoint_symbol,
                ship_type,
                price,
                agent_symbol,
                "timestamp"
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::ship_type[],
                $3::integer[],
                $4::character varying[],
                $5::timestamp[]
            )
            ON CONFLICT (waypoint_symbol, ship_type, agent_symbol, "timestamp") DO NOTHING
            "#,
            &waypoint_symbols,
            &ship_types as &[models::ShipType],
            &prices,
            &agent_symbols,
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
    ) -> crate::Result<PaginatedResult<ShipyardTransaction>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
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
                    ShipyardTransaction,
                    r#"
                        SELECT
                            id,
                            waypoint_symbol,
                            ship_type as "ship_type: models::ShipType",
                            price,
                            agent_symbol,
                            "timestamp"
                        FROM shipyard_transaction
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
                        FROM shipyard_transaction
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
            ShipyardTransaction,
            r#"
                SELECT
                    id,
                    waypoint_symbol,
                    ship_type as "ship_type: models::ShipType",
                    price,
                    agent_symbol,
                    "timestamp"
                FROM shipyard_transaction
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
                DELETE FROM shipyard_transaction
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
