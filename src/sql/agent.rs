use super::{
    sql_models::{self, Agent},
    DatabaseConnector, DbPool,
};

impl From<space_traders_client::models::Agent> for Agent {
    fn from(item: space_traders_client::models::Agent) -> Agent {
        Agent {
            account_id: item.account_id,
            symbol: item.symbol,
            headquarters: item.headquarters,
            credits: item.credits,
            starting_faction: item.starting_faction,
            ship_count: item.ship_count,
            created_at:  sqlx::types::time::PrimitiveDateTime::MIN,
        }
    }
}

impl sql_models::Agent {
  async fn get_last(database_pool: &DbPool) -> sqlx::Result<Vec<Agent>> {
    let result = sqlx::query_as! {
        Agent,
        r#"
        SELECT DISTINCT ON (symbol) symbol, account_id, headquarters, credits, starting_faction, ship_count, created_at
        FROM agent
        ORDER BY  symbol ASC, created_at DESC
        "#
    }
    .fetch_all(&database_pool.database_pool)
    .await;

    result
  }

   pub async fn get_last_by_symbol(database_pool: &DbPool, symbol: &str) -> sqlx::Result<Agent> {
    let result = sqlx::query_as! {
        Agent,
        r#"
        SELECT DISTINCT ON (symbol) symbol, account_id, headquarters, credits, starting_faction, ship_count, created_at
        FROM agent WHERE symbol = $1
        ORDER BY  symbol ASC, created_at DESC
        LIMIT 1
        "#,
        symbol
    }
    .fetch_one(&database_pool.database_pool)
    .await;

    result
  }
}

impl DatabaseConnector<Agent> for sql_models::Agent {
    async fn insert(database_pool: &DbPool, item: &Agent) -> sqlx::Result<()> {
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

    async fn insert_bulk(database_pool: &DbPool, items: &Vec<Agent>) -> sqlx::Result<()> {
        let (
            (
              (account_ids,symbols),
              (creditss,ship_counts)
            ),(
              (headquarterss, starting_factions),
              (_,_)
            )
         ): (((Vec<_>, Vec<_>),(Vec<Result<i32, std::num::TryFromIntError>>, Vec<_>)),((Vec<_>, Vec<_>),(Vec<_>, Vec<_>)))=items.iter().map(|a|{
          (
            (
              (a.account_id.clone(),a.symbol.clone()),
              (a.credits.try_into(),a.ship_count)
            ),(
              (a.headquarters.clone(), a.starting_faction.clone()),
              ((),())
            )
         )
      }).unzip();

      let creditss=creditss.into_iter().filter_map(|c|c.ok()).collect::<Vec<_>>();

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

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<Agent>> {
        sqlx::query_as!(
            Agent,
            r#"
                SELECT symbol, account_id, headquarters, credits, starting_faction, ship_count, created_at FROM agent
            "#
        ) 
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
