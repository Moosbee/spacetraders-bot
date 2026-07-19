use itertools::Itertools;
use tracing::instrument;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBAgent")]
pub struct Agent {
    pub id: i64,
    pub symbol: String,
    pub account_id: Option<String>,
    pub headquarters: String,
    pub credits: i64,
    pub starting_faction: String,
    pub ship_count: i32,
    #[allow(dead_code)]
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
}

impl From<space_traders_client::models::Agent> for Agent {
    fn from(item: space_traders_client::models::Agent) -> Agent {
        Agent {
            id: 0,
            account_id: Some(item.account_id),
            symbol: item.symbol,
            headquarters: item.headquarters,
            credits: item.credits,
            starting_faction: item.starting_faction,
            ship_count: item.ship_count,
            created_at: sqlx::types::chrono::DateTime::<chrono::Utc>::MIN_UTC,
        }
    }
}

impl From<space_traders_client::models::PublicAgent> for Agent {
    fn from(item: space_traders_client::models::PublicAgent) -> Agent {
        Agent {
            id: 0,
            account_id: None,
            symbol: item.symbol,
            headquarters: item.headquarters,
            credits: item.credits,
            starting_faction: item.starting_faction,
            ship_count: item.ship_count,
            created_at: sqlx::types::chrono::DateTime::<chrono::Utc>::MIN_UTC,
        }
    }
}

impl Agent {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Agent>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let agents = sqlx::query_as!(
                    Agent,
                    r#"
                        SELECT DISTINCT ON (symbol)
                          id,
                          symbol,
                          account_id,
                          headquarters,
                          credits,
                          starting_faction,
                          ship_count,
                          created_at
                        FROM agent
                        ORDER BY symbol ASC, created_at DESC
                        LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;

                Ok(agents)
            },
            || async move {
                let agents = sqlx::query_as!(
                    Agent,
                    r#"
                        SELECT DISTINCT ON (symbol)
                          id,
                          symbol,
                          account_id,
                          headquarters,
                          credits,
                          starting_faction,
                          ship_count,
                          created_at
                        FROM agent
                        ORDER BY symbol ASC, created_at DESC
                    "#,
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;

                Ok(agents)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(DISTINCT symbol) as "count!"
                        FROM agent
                    "#,
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;

                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_symbol(
        database_pool: &DbPool,
        symbol: &str,
    ) -> crate::Result<Option<Agent>> {
        let erg= sqlx::query_as! {
        Agent,
        r#"
        SELECT DISTINCT ON (symbol) id, symbol, account_id, headquarters, credits, starting_faction, ship_count, created_at
        FROM agent WHERE symbol = $1
        ORDER BY  symbol ASC, created_at DESC
        LIMIT 1
        "#,
        symbol
    }
    .fetch_optional(database_pool.get_cache_pool())
    .await?;

        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_symbol(
        database_pool: &DbPool,
        symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Agent>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let agents = sqlx::query_as!(
                    Agent,
                    r#"
                        SELECT 
                          id,
                          symbol,
                          account_id,
                          headquarters,
                          credits,
                          starting_faction,
                          ship_count,
                          created_at
                        FROM agent
                        WHERE symbol = $1
                        ORDER BY symbol ASC, created_at DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;

                Ok(agents)
            },
            || async move {
                let agents = sqlx::query_as!(
                    Agent,
                    r#"
                        SELECT 
                          id,
                          symbol,
                          account_id,
                          headquarters,
                          credits,
                          starting_faction,
                          ship_count,
                          created_at
                        FROM agent
                        WHERE symbol = $1
                        ORDER BY symbol ASC, created_at DESC
                    "#,
                    symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;

                Ok(agents)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM agent
                        WHERE symbol = $1
                    "#,
                    symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;

                Ok(count.count)
            },
        )
        .await
    }
}

impl DatabaseConnectorAsync for Agent {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &DbPool, item: &Agent) -> crate::Result<Self::ID> {
        let inserted = sqlx::query_as!(
            Agent,
            r#"
                INSERT INTO agent (
                  symbol,
                  account_id,
                  headquarters,
                  credits,
                  starting_faction,
                  ship_count
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING
                  id,
                  symbol,
                  account_id,
                  headquarters,
                  credits,
                  starting_faction,
                  ship_count,
                  created_at
            "#,
            item.symbol,
            item.account_id,
            item.headquarters,
            item.credits,
            item.starting_faction,
            item.ship_count
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        database_pool
            .agent_broadcast_channel
            .0
            .send(inserted.clone())
            .unwrap();

        Ok(inserted.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &Agent) -> crate::Result<()> {
        let _id = Self::insert_new(database_pool, item).await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &Agent) -> crate::Result<()> {
        let agent_id = item.id as i32;

        sqlx::query!(
            r#"
                UPDATE agent
                SET
                  symbol = $1,
                  account_id = $2,
                  headquarters = $3,
                  credits = $4,
                  starting_faction = $5,
                  ship_count = $6
                WHERE id = $7
            "#,
            item.symbol,
            item.account_id,
            item.headquarters,
            item.credits,
            item.starting_faction,
            item.ship_count,
            agent_id
        )
        .execute(&database_pool.database_pool)
        .await?;

        database_pool
            .agent_broadcast_channel
            .0
            .send(item.clone())
            .unwrap();

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[Agent]) -> crate::Result<()> {
        let (account_ids, symbols, credits, ship_counts, headquarters, starting_factions): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = items
            .iter()
            .map(|a| {
                (
                    a.account_id.clone(),
                    a.symbol.clone(),
                    a.credits,
                    a.ship_count,
                    a.headquarters.clone(),
                    a.starting_faction.clone(),
                )
            })
            .multiunzip();

        let inserted = sqlx::query_as!(
            Agent,
            r#"
                INSERT INTO agent (
                  symbol,
                  account_id,
                  headquarters,
                  credits,
                  starting_faction,
                  ship_count
                )
                SELECT * FROM UNNEST(
                  $1::character varying[],
                  $2::character varying[],
                  $3::character varying[],
                  $4::bigint[],
                  $5::character varying[],
                  $6::integer[]
                )
                RETURNING
                  id,
                  symbol,
                  account_id,
                  headquarters,
                  credits,
                  starting_faction,
                  ship_count,
                  created_at
            "#,
            &symbols,
            &account_ids as &[Option<String>],
            &headquarters,
            &credits,
            &starting_factions,
            &ship_counts
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        for item in inserted {
            database_pool.agent_broadcast_channel.0.send(item).unwrap();
        }

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Agent>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let agents = sqlx::query_as!(
                    Agent,
                    r#"
                        SELECT
                          id,
                          symbol,
                          account_id,
                          headquarters,
                          credits,
                          starting_faction,
                          ship_count,
                          created_at
                        FROM agent
                        ORDER BY id ASC
                        LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;

                Ok(agents)
            },
            || async move {
                let agents = sqlx::query_as!(
                    Agent,
                    r#"
                        SELECT
                          id,
                          symbol,
                          account_id,
                          headquarters,
                          credits,
                          starting_faction,
                          ship_count,
                          created_at
                        FROM agent
                        ORDER BY id ASC
                    "#,
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;

                Ok(agents)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM agent
                    "#,
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
        let agent_id = *id as i32;

        let agent = sqlx::query_as!(
            Agent,
            r#"
                SELECT
                  id,
                  symbol,
                  account_id,
                  headquarters,
                  credits,
                  starting_faction,
                  ship_count,
                  created_at
                FROM agent
                WHERE id = $1
                LIMIT 1
            "#,
            agent_id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;

        Ok(agent)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        let agent_id = *id as i32;

        sqlx::query!(
            r#"
                DELETE FROM agent
                WHERE id = $1
            "#,
            agent_id
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = id;
    }
}
