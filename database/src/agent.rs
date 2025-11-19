use itertools::Itertools;
use tracing::instrument;

use super::{DatabaseConnector, DbPool};

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
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_last(database_pool: &DbPool) -> crate::Result<Vec<Agent>> {
        let erg= sqlx::query_as! {
        Agent,
        r#"
        SELECT DISTINCT ON (symbol) id, symbol, account_id, headquarters, credits, starting_faction, ship_count, created_at
        FROM agent
        ORDER BY  symbol ASC, created_at DESC
        "#
    }
    .fetch_all(database_pool.get_cache_pool())
    .await?;

        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
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

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_symbol(database_pool: &DbPool, symbol: &str) -> crate::Result<Vec<Agent>> {
        let erg = sqlx::query_as!(
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
                FROM agent WHERE symbol = $1
                ORDER BY  symbol ASC, created_at DESC
            "#,
            symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnector<Agent> for Agent {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &DbPool, item: &Agent) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO agent (symbol, account_id, headquarters, credits, starting_faction, ship_count)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            item.symbol, item.account_id, item.headquarters, item.credits, item.starting_faction, item.ship_count
        ).execute(&database_pool.database_pool).await?;

        database_pool
            .agent_broadcast_channel
            .0
            .send(item.clone())
            .unwrap();

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[Agent]) -> crate::Result<()> {
        let (account_ids, symbols, creditss, ship_counts, headquarterss, starting_factions): (
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
                    a.credits.try_into(),
                    a.ship_count,
                    a.headquarters.clone(),
                    a.starting_faction.clone(),
                )
            })
            .multiunzip();

        let creditss = creditss
            .into_iter()
            .filter_map(|c| c.ok())
            .collect::<Vec<_>>();

        sqlx::query!(
        r#"
            INSERT INTO agent (symbol, account_id, headquarters, credits, starting_faction, ship_count)
            select * from UNNEST($1::character varying[], $2::character varying[], $3::character varying[], $4::integer[], $5::character varying[], $6::integer[])
        "#,
        &symbols,
        &account_ids as &[Option<String>],
        &headquarterss,
        &creditss,
        &starting_factions,
        &ship_counts
    ).execute(&database_pool.database_pool).await?;

        for item in items {
            database_pool
                .agent_broadcast_channel
                .0
                .send(item.clone())
                .unwrap();
        }

        // let mm:(Vec<_>,Vec<_>)=ag.iter().unzip();

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<Agent>> {
        let erg= sqlx::query_as!(
            Agent,
            r#"
                SELECT id, symbol, account_id, headquarters, credits, starting_faction, ship_count, created_at FROM agent
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
