use super::{
    sql_models::{DatabaseConnector, Waypoint},
    DbPool,
};

impl From<&space_traders_client::models::Waypoint> for super::sql_models::Waypoint {
    fn from(value: &space_traders_client::models::Waypoint) -> Self {
        super::sql_models::Waypoint {
            symbol: value.symbol.clone(),
            system_symbol: value.system_symbol.clone(),
            ..Default::default()
        }
    }
}

impl Default for Waypoint {
    fn default() -> Self {
        Self {
            symbol: "".to_string(),
            system_symbol: "".to_string(),
            created_at: sqlx::types::time::PrimitiveDateTime::MIN,
        }
    }
}

impl DatabaseConnector<Waypoint> for Waypoint {
    async fn insert(database_pool: &DbPool, item: &Waypoint) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO waypoint (symbol, system_symbol)
                VALUES ($1, $2)
                ON CONFLICT (symbol) DO UPDATE SET system_symbol = EXCLUDED.system_symbol
            "#,
            item.symbol,
            item.system_symbol
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &Vec<Waypoint>) -> sqlx::Result<()> {
        let (m_symbols, f_symbols): (Vec<String>, Vec<String>) = items
            .iter()
            .map(|w| (w.symbol.clone(), w.system_symbol.clone()))
            .unzip();

        sqlx::query!(
            r#"
            INSERT INTO waypoint (symbol, system_symbol)
            SELECT * FROM UNNEST($1::character varying[], $2::character varying[])
            ON CONFLICT (symbol) DO UPDATE SET system_symbol = EXCLUDED.system_symbol
        "#,
            &m_symbols,
            &f_symbols
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<Waypoint>> {
        sqlx::query_as!(
            Waypoint,
            r#"
                SELECT symbol, system_symbol, created_at
                FROM waypoint
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
