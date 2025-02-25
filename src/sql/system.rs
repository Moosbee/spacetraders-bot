use space_traders_client::models;

use super::DatabaseConnector;

impl From<models::System> for super::sql_models::System {
    fn from(system: models::System) -> Self {
        super::sql_models::System {
            symbol: system.symbol,
            sector_symbol: system.sector_symbol,
            system_type: system.r#type,
            x: system.x,
            y: system.y,
        }
    }
}

impl DatabaseConnector<super::sql_models::System> for super::sql_models::System {
    async fn insert(
        database_pool: &super::DbPool,
        item: &super::sql_models::System,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO system (
                    symbol,
                    sector_symbol,
                    system_type,
                    x,
                    y
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (symbol) DO UPDATE
                SET sector_symbol = EXCLUDED.sector_symbol,
                    system_type = EXCLUDED.system_type,
                    x = EXCLUDED.x,
                    y = EXCLUDED.y
            "#,
            item.symbol,
            item.sector_symbol,
            item.system_type as models::SystemType,
            item.x,
            item.y
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &Vec<super::sql_models::System>,
    ) -> sqlx::Result<()> {
        let (symbols, sector_symbols, system_types, xs, ys): (
            Vec<String>,
            Vec<String>,
            Vec<models::SystemType>,
            Vec<i32>,
            Vec<i32>,
        ) = itertools::multiunzip(items.iter().map(|s| {
            (
                s.symbol.clone(),
                s.sector_symbol.clone(),
                s.system_type,
                s.x,
                s.y,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO system (
                symbol,
                sector_symbol,
                system_type,
                x,
                y
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::system_type[],
                $4::integer[],
                $5::integer[]
            )
            ON CONFLICT (symbol) DO UPDATE
            SET sector_symbol = EXCLUDED.sector_symbol,
                system_type = EXCLUDED.system_type,
                x = EXCLUDED.x,
                y = EXCLUDED.y
            "#,
            &symbols,
            &sector_symbols,
            &system_types as &[models::SystemType],
            &xs,
            &ys
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn get_all(
        database_pool: &super::DbPool,
    ) -> sqlx::Result<Vec<super::sql_models::System>> {
        sqlx::query_as!(
            super::sql_models::System,
            r#"
            SELECT 
                symbol,
                sector_symbol,
                system_type as "system_type: models::SystemType",
                x,
                y
            FROM system
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl super::sql_models::System {
    pub async fn get_by_id(
        database_pool: &super::DbPool,
        id: &String,
    ) -> sqlx::Result<Option<super::sql_models::System>> {
        sqlx::query_as!(
            super::sql_models::System,
            r#"
        SELECT 
            symbol,
            sector_symbol,
            system_type as "system_type: models::SystemType",
            x,
            y
        FROM system
        WHERE symbol = $1
        LIMIT 1
        "#,
            id
        )
        .fetch_optional(&database_pool.database_pool)
        .await
    }
}
