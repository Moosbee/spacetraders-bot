use chrono::{DateTime, Utc};
use tracing::instrument;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(
    Clone,
    Default,
    Debug,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    async_graphql::SimpleObject,
)]
#[graphql(name = "DBJumpGateConnection")]
pub struct JumpGateConnection {
    pub id: i64,
    pub from: String,
    pub to: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl JumpGateConnection {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_all_from(
        database_pool: &DbPool,
        from: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<JumpGateConnection>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    JumpGateConnection,
                    r#"
                SELECT
                  id,
                  waypoint_from as "from",
                  waypoint_to as "to",
                  created_at,
                  updated_at
                FROM jump_gate_connections
                WHERE waypoint_from = $1
                ORDER BY updated_at DESC, id DESC
                LIMIT $2 OFFSET $3
              "#,
                    from,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    JumpGateConnection,
                    r#"
                SELECT
                  id,
                  waypoint_from as "from",
                  waypoint_to as "to",
                  created_at,
                  updated_at
                FROM jump_gate_connections
                WHERE waypoint_from = $1
                ORDER BY updated_at DESC, id DESC
              "#,
                    from
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM jump_gate_connections
                    WHERE waypoint_from = $1
                    "#,
                    from
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_all_from_system(
        database_pool: &DbPool,
        system_from: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<JumpGateConnection>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    JumpGateConnection,
                    r#"
                SELECT
                  id,
                  waypoint_from as "from",
                  waypoint_to as "to",
                  created_at,
                  updated_at
                FROM jump_gate_connections
                WHERE waypoint_from LIKE ($1 || '-%')
                ORDER BY updated_at DESC, id DESC
                LIMIT $2 OFFSET $3
              "#,
                    system_from,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    JumpGateConnection,
                    r#"
                SELECT
                  id,
                  waypoint_from as "from",
                  waypoint_to as "to",
                  created_at,
                  updated_at
                FROM jump_gate_connections
                WHERE waypoint_from LIKE ($1 || '-%')
                ORDER BY updated_at DESC, id DESC
              "#,
                    system_from
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM jump_gate_connections
                    WHERE waypoint_from LIKE ($1 || '-%')
                    "#,
                    system_from
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }
}

impl DatabaseConnectorAsync for JumpGateConnection {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &DbPool,
        item: &JumpGateConnection,
    ) -> crate::Result<Self::ID> {
        let record = sqlx::query!(
            r#"
                INSERT INTO jump_gate_connections (
                  waypoint_from,
                  waypoint_to,
                  updated_at
                )
                VALUES ($1, $2, NOW())
                ON CONFLICT (waypoint_from, waypoint_to) DO UPDATE SET
                  updated_at = NOW()
                RETURNING id;
            "#,
            &item.from,
            &item.to
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(record.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &JumpGateConnection) -> crate::Result<()> {
        let _ = Self::insert_new(database_pool, item).await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &JumpGateConnection) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &DbPool,
        items: &[JumpGateConnection],
    ) -> crate::Result<()> {
        // for item in items.iter() {
        //     Self::insert(database_pool, item).await?;
        // }

        let (waypoints_from, waypoints_to): (Vec<String>, Vec<String>) =
            itertools::multiunzip(items.iter().map(|jg| (jg.from.clone(), jg.to.clone())));

        sqlx::query!(
            r#"
            INSERT INTO jump_gate_connections (
                waypoint_from,
                waypoint_to,
                updated_at
            )
            SELECT from_wp, to_wp, NOW() FROM UNNEST(
                $1::character varying[],
                $2::character varying[]
            ) AS t(from_wp, to_wp)
            ON CONFLICT (waypoint_from, waypoint_to) DO UPDATE
            SET updated_at = NOW();
            "#,
            &waypoints_from,
            &waypoints_to
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<JumpGateConnection>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    JumpGateConnection,
                    r#"
                        SELECT
                          id,
                          waypoint_from as "from",
                          waypoint_to as "to",
                          created_at,
                          updated_at
                        FROM jump_gate_connections
                        ORDER BY updated_at DESC, id DESC
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
                    JumpGateConnection,
                    r#"
                        SELECT
                          id,
                          waypoint_from as "from",
                          waypoint_to as "to",
                          created_at,
                          updated_at
                        FROM jump_gate_connections
                        ORDER BY updated_at DESC, id DESC
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
                    FROM jump_gate_connections
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
            JumpGateConnection,
            r#"
                SELECT
                  id,
                  waypoint_from as "from",
                  waypoint_to as "to",
                  created_at,
                  updated_at
                FROM jump_gate_connections
                WHERE id = $1
                LIMIT 1
            "#,
            *id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(item)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM jump_gate_connections
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
