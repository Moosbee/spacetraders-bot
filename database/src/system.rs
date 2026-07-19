use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBSystem")]
pub struct System {
    pub symbol: String,
    pub constellation: Option<String>,
    pub sector_symbol: String,
    pub system_type: models::SystemType,
    pub x: i32,
    pub y: i32,
    pub population_disabled: bool,
    // pub factions: Vec<String>,
}

impl From<System> for (i32, i32) {
    fn from(value: System) -> Self {
        (value.x, value.y)
    }
}
impl From<&System> for (i32, i32) {
    fn from(value: &System) -> Self {
        (value.x, value.y)
    }
}

impl From<&models::System> for System {
    fn from(system: &models::System) -> Self {
        System {
            constellation: system.constellation.clone(),
            population_disabled: false,
            symbol: system.symbol.clone(),
            sector_symbol: system.sector_symbol.clone(),
            system_type: system.r#type,
            x: system.x,
            y: system.y,
        }
    }
}

impl System {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn set_population_disabled_led(
        database_pool: &DbPool,
        symbol: &str,
        disabled: bool,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
            UPDATE system
            SET population_disabled = $1
            WHERE symbol = $2
            "#,
            disabled,
            symbol
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }
}

impl DatabaseConnectorAsync for System {
    type ID = String;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &DbPool, item: &System) -> crate::Result<Self::ID> {
        Self::upsert(database_pool, item).await?;
        Ok(item.symbol.clone())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &System) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO system (
                    symbol,
                    sector_symbol,
                    constellation,
                    population_disabled,
                    system_type,
                    x,
                    y
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (symbol) DO UPDATE
                SET sector_symbol = EXCLUDED.sector_symbol,
                    constellation = EXCLUDED.constellation,
                    population_disabled = EXCLUDED.population_disabled,
                    system_type = EXCLUDED.system_type,
                    x = EXCLUDED.x,
                    y = EXCLUDED.y
            "#,
            item.symbol,
            item.sector_symbol,
            item.constellation,
            item.population_disabled,
            item.system_type as models::SystemType,
            item.x,
            item.y
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &System) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[System]) -> crate::Result<()> {
        let (symbols, constellation, population_disableds, sector_symbols, system_types, xs, ys): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|s| {
            (
                s.symbol.clone(),
                s.constellation.clone(),
                s.population_disabled,
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
                constellation,
                population_disabled,
                sector_symbol,
                system_type,
                x,
                y
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::boolean[],
                $4::character varying[],
                $5::system_type[],
                $6::integer[],
                $7::integer[]
            )
            ON CONFLICT (symbol) DO UPDATE
            SET sector_symbol = EXCLUDED.sector_symbol,
                constellation = EXCLUDED.constellation,
                population_disabled = EXCLUDED.population_disabled,
                system_type = EXCLUDED.system_type,
                x = EXCLUDED.x,
                y = EXCLUDED.y
            "#,
            &symbols,
            &constellation as &[Option<String>],
            &population_disableds,
            &sector_symbols,
            &system_types as &[models::SystemType],
            &xs,
            &ys
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<System>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    System,
                    r#"
                    SELECT 
                        symbol,
                        constellation,
                        population_disabled,
                        sector_symbol,
                        system_type as "system_type: models::SystemType",
                        x,
                        y
                    FROM system
                    ORDER BY symbol ASC
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
                    System,
                    r#"
                    SELECT 
                        symbol,
                        constellation,
                        population_disabled,
                        sector_symbol,
                        system_type as "system_type: models::SystemType",
                        x,
                        y
                    FROM system
                    ORDER BY symbol ASC
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
                    FROM system
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
        let erg = sqlx::query_as!(
            System,
            r#"
            SELECT
                symbol,
                constellation,
                population_disabled,
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
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM system
            WHERE symbol = $1
            "#,
            id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.symbol = id;
    }
}
