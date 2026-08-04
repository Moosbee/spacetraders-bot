use chrono::{DateTime, Utc};
use tracing::instrument;

use super::DbPool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShipEvent {
    pub id: i64,
    pub ship_symbol: String,
    pub event_type: String,
    pub event_data: sqlx::types::Json<serde_json::Value>,
    pub state_before: sqlx::types::Json<serde_json::Value>,
    pub state_after: Option<sqlx::types::Json<serde_json::Value>>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl ShipEvent {
    #[instrument(level = "trace", skip(database_pool, item))]
    pub async fn insert(database_pool: &DbPool, item: &ShipEvent) -> crate::Result<i64> {
        let id = sqlx::query!(
            r#"
                INSERT INTO ship_event (
                  ship_symbol,
                  event_type,
                  event_data,
                  state_before,
                  state_after,
                  duration_ms
                )
                VALUES ($1, $2, $3::jsonb, $4::jsonb, $5::jsonb, $6)
                RETURNING id;
            "#,
            &item.ship_symbol,
            &item.event_type,
            &item.event_data as &sqlx::types::Json<serde_json::Value>,
            &item.state_before as &sqlx::types::Json<serde_json::Value>,
            &item.state_after as &Option<sqlx::types::Json<serde_json::Value>>,
            &item.duration_ms as &Option<i64>,
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        Ok(id.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship(
        database_pool: &DbPool,
        ship_symbol: &str,
        limit: i64,
        offset: i64,
    ) -> crate::Result<Vec<ShipEvent>> {
        let items = sqlx::query_as!(
            ShipEvent,
            r#"
                SELECT
                  id,
                  ship_symbol,
                  event_type,
                  event_data as "event_data: sqlx::types::Json<serde_json::Value>",
                  state_before as "state_before: sqlx::types::Json<serde_json::Value>",
                  state_after as "state_after: sqlx::types::Json<serde_json::Value>",
                  duration_ms,
                  created_at
                FROM ship_event
                WHERE ship_symbol = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
            "#,
            ship_symbol,
            limit,
            offset
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(items)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship_and_type(
        database_pool: &DbPool,
        ship_symbol: &str,
        event_type: &str,
        limit: i64,
        offset: i64,
    ) -> crate::Result<Vec<ShipEvent>> {
        let items = sqlx::query_as!(
            ShipEvent,
            r#"
                SELECT
                  id,
                  ship_symbol,
                  event_type,
                  event_data as "event_data: sqlx::types::Json<serde_json::Value>",
                  state_before as "state_before: sqlx::types::Json<serde_json::Value>",
                  state_after as "state_after: sqlx::types::Json<serde_json::Value>",
                  duration_ms,
                  created_at
                FROM ship_event
                WHERE ship_symbol = $1 AND event_type = $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
            "#,
            ship_symbol,
            event_type,
            limit,
            offset
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(items)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_id(database_pool: &DbPool, id: i64) -> crate::Result<Option<ShipEvent>> {
        let item = sqlx::query_as!(
            ShipEvent,
            r#"
                SELECT
                  id,
                  ship_symbol,
                  event_type,
                  event_data as "event_data: sqlx::types::Json<serde_json::Value>",
                  state_before as "state_before: sqlx::types::Json<serde_json::Value>",
                  state_after as "state_after: sqlx::types::Json<serde_json::Value>",
                  duration_ms,
                  created_at
                FROM ship_event
                WHERE id = $1
            "#,
            id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(item)
    }
}
