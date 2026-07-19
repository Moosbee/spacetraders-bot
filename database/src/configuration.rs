use crate::DatabaseConnectorAsync;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Configuration {
    key: String,
    value: sqlx::types::Json<serde_json::Value>,
    #[allow(dead_code)]
    pub updated_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    #[allow(dead_code)]
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
}

impl Configuration {
    pub fn new(key: String, value: serde_json::Value) -> Configuration {
        Configuration {
            key,
            value: sqlx::types::Json(value),
            updated_at: sqlx::types::chrono::Utc::now(),
            created_at: sqlx::types::chrono::Utc::now(),
        }
    }

    pub async fn get_by_key(
        database_pool: &crate::DbPool,
        key: &str,
    ) -> crate::Result<Option<Self>> {
        let erg = sqlx::query_as!(
            Configuration,
            r#"
                SELECT
                    key,
                    value as "value!: sqlx::types::Json<serde_json::Value>",
                    updated_at,
                    created_at
                FROM configuration
                WHERE key = $1
                LIMIT 1
            "#,
            key
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;

        Ok(erg)
    }

    pub async fn get_agent_token(database_pool: &crate::DbPool) -> crate::Result<Option<String>> {
        let erg = Self::get_by_key(database_pool, "agent_token").await?;
        Ok(erg.and_then(|c| c.value.as_str().map(|f| f.to_string())))
    }

    pub async fn set_agent_token(database_pool: &crate::DbPool, token: &str) -> crate::Result<()> {
        let config = Self::new(
            "agent_token".to_string(),
            serde_json::Value::String(token.to_string()),
        );
        Self::upsert(database_pool, &config).await
    }
}

impl DatabaseConnectorAsync for Configuration {
    type ID = String; // key

    #[tracing::instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &crate::DbPool, item: &Self) -> crate::Result<Self::ID> {
        sqlx::query!(
            r#"
              INSERT INTO public.configuration (
                key,
                value,
                updated_at,
                created_at
              ) VALUES (
                $1,
                $2,
                $3,
                $4
              )
              ON CONFLICT (key) DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = EXCLUDED.updated_at;
            "#,
            &item.key,
            &item.value as &sqlx::types::Json<serde_json::Value>,
            item.updated_at,
            item.created_at,
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(item.key.clone())
    }

    #[tracing::instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &crate::DbPool, item: &Self) -> crate::Result<()> {
        let _ = Self::insert_new(database_pool, item).await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &crate::DbPool, item: &Self) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[tracing::instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &crate::DbPool, items: &[Self]) -> crate::Result<()> {
        let (keys, values, updated_ats, created_ats): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
            itertools::Itertools::multiunzip(
                items
                    .iter()
                    .cloned()
                    .map(|c| (c.key, c.value.0, c.updated_at, c.created_at)),
            );

        sqlx::query!(
            r#"
              INSERT INTO public.configuration (
                key,
                value,
                updated_at,
                created_at
              )
              SELECT * FROM UNNEST(
                $1::character varying[],
                $2::jsonb[],
                $3::timestamptz[],
                $4::timestamptz[]
              )
              ON CONFLICT (key) DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = EXCLUDED.updated_at;
            "#,
            &keys as &[String],
            &values as &[serde_json::Value],
            &updated_ats as &[sqlx::types::chrono::DateTime<chrono::Utc>],
            &created_ats as &[sqlx::types::chrono::DateTime<chrono::Utc>],
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &crate::DbPool,
        query: crate::PaginatedQuery,
    ) -> crate::Result<crate::PaginatedResult<Self>> {
        crate::run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    Configuration,
                    r#"
                        SELECT
                            key,
                            value as "value!: sqlx::types::Json<serde_json::Value>",
                            updated_at,
                            created_at
                        FROM configuration
                        LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    Configuration,
                    r#"
                        SELECT
                            key,
                            value as "value!: sqlx::types::Json<serde_json::Value>",
                            updated_at,
                            created_at
                        FROM configuration
                    "#
                )
                .fetch_all(&database_pool.database_pool)
                .await?;

                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM configuration
                    "#
                )
                .fetch_one(&database_pool.database_pool)
                .await?;

                Ok(count.count)
            },
        )
        .await
    }

    #[tracing::instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_by_id(
        database_pool: &crate::DbPool,
        id: &Self::ID,
    ) -> crate::Result<Option<Self>> {
        let erg = sqlx::query_as!(
            Configuration,
            r#"
                SELECT
                    key,
                    value as "value!: sqlx::types::Json<serde_json::Value>",
                    updated_at,
                    created_at
                FROM configuration
                WHERE key = $1
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;

        Ok(erg)
    }

    #[tracing::instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &crate::DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM configuration
            WHERE key = $1
            "#,
            id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.key = id;
    }
}
