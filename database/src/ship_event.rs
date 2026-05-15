use chrono::{DateTime, Utc};
use tracing::instrument;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

type JsonPayload = sqlx::types::Json<serde_json::Value>;

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "DBShipEvent")]
pub struct ShipEvent {
    pub id: i64,
    pub ship_symbol: String,
    pub event_kind: String,
    pub event_name: String,
    pub event_phase: String,
    pub correlation_id: String,
    #[graphql(skip)]
    pub payload: JsonPayload,
    pub before_ship_state_id: i64,
    pub after_ship_state_id: i64,
    pub created_at: DateTime<Utc>,
}

impl ShipEvent {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship(
        database_pool: &DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipEvent>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipEvent,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          event_kind,
                          event_name,
                          event_phase,
                          correlation_id,
                          payload as "payload: JsonPayload",
                          before_ship_state_id,
                          after_ship_state_id,
                          created_at
                        FROM ship_event
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
                    ShipEvent,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          event_kind,
                          event_name,
                          event_phase,
                          correlation_id,
                          payload as "payload: JsonPayload",
                          before_ship_state_id,
                          after_ship_state_id,
                          created_at
                        FROM ship_event
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
                        FROM ship_event
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

impl DatabaseConnectorAsync for ShipEvent {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &DbPool, item: &ShipEvent) -> crate::Result<Self::ID> {
        let inserted = sqlx::query!(
            r#"
                INSERT INTO ship_event (
                    ship_symbol,
                    event_kind,
                    event_name,
                    event_phase,
                    correlation_id,
                    payload,
                    before_ship_state_id,
                    after_ship_state_id
                )
                VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, $8)
                RETURNING id
            "#,
            item.ship_symbol,
            item.event_kind,
            item.event_name,
            item.event_phase,
            item.correlation_id,
            &item.payload as &JsonPayload,
            item.before_ship_state_id,
            item.after_ship_state_id,
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        Ok(inserted.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &ShipEvent) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO ship_event (
                    ship_symbol,
                    event_kind,
                    event_name,
                    event_phase,
                    correlation_id,
                    payload,
                    before_ship_state_id,
                    after_ship_state_id
                )
                VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, $8)
                ON CONFLICT (id) DO NOTHING
            "#,
            item.ship_symbol,
            item.event_kind,
            item.event_name,
            item.event_phase,
            item.correlation_id,
            &item.payload as &JsonPayload,
            item.before_ship_state_id,
            item.after_ship_state_id,
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &ShipEvent) -> crate::Result<()> {
        sqlx::query!(
            r#"
                UPDATE ship_event
                SET
                    ship_symbol = $1,
                    event_kind = $2,
                    event_name = $3,
                    event_phase = $4,
                    correlation_id = $5,
                    payload = $6::jsonb,
                    before_ship_state_id = $7,
                    after_ship_state_id = $8
                WHERE id = $9
            "#,
            item.ship_symbol,
            item.event_kind,
            item.event_name,
            item.event_phase,
            item.correlation_id,
            &item.payload as &JsonPayload,
            item.before_ship_state_id,
            item.after_ship_state_id,
            item.id,
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[ShipEvent]) -> crate::Result<()> {
        for item in items {
            Self::upsert(database_pool, item).await?;
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipEvent>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipEvent,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          event_kind,
                          event_name,
                          event_phase,
                          correlation_id,
                          payload as "payload: JsonPayload",
                          before_ship_state_id,
                          after_ship_state_id,
                          created_at
                        FROM ship_event
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
                    ShipEvent,
                    r#"
                        SELECT
                          id,
                          ship_symbol,
                          event_kind,
                          event_name,
                          event_phase,
                          correlation_id,
                          payload as "payload: JsonPayload",
                          before_ship_state_id,
                          after_ship_state_id,
                          created_at
                        FROM ship_event
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
                        FROM ship_event
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
        let event = sqlx::query_as!(
            ShipEvent,
            r#"
                SELECT
                  id,
                  ship_symbol,
                  event_kind,
                  event_name,
                  event_phase,
                  correlation_id,
                  payload as "payload: JsonPayload",
                  before_ship_state_id,
                  after_ship_state_id,
                  created_at
                FROM ship_event
                WHERE id = $1
                LIMIT 1
            "#,
            *id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(event)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM ship_event
                WHERE id = $1
            "#,
            id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = id;
    }
}
