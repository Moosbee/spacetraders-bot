use space_traders_client::models;
use tracing::instrument;

use super::{run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash, async_graphql::SimpleObject)]
#[graphql(name = "DBModuleInfo")]
pub struct ModuleInfo {
    pub symbol: models::ship_module::Symbol,
    pub name: String,
    pub description: String,
    pub capacity: Option<i32>,
    pub range: Option<i32>,
    pub power_required: Option<i32>,
    pub crew_required: Option<i32>,
    pub slots_required: Option<i32>,
}

impl From<models::ship_module::ShipModule> for ModuleInfo {
    fn from(value: models::ship_module::ShipModule) -> Self {
        ModuleInfo {
            symbol: value.symbol,
            name: value.name,
            description: value.description,
            capacity: value.capacity,
            range: value.range,
            power_required: value.requirements.power,
            crew_required: value.requirements.crew,
            slots_required: value.requirements.slots,
        }
    }
}

impl ModuleInfo {
    pub async fn get_by_id(
        database_pool: &super::DbPool,
        symbol: &models::ship_module::Symbol,
    ) -> crate::Result<ModuleInfo> {
        let erg = sqlx::query_as!(
            ModuleInfo,
            r#"
                SELECT
                    symbol as "symbol: models::ship_module::Symbol",
                    name,
                    description,
                    range,
                    capacity,
                    power_required,
                    crew_required,
                    slots_required
                FROM module_info
                WHERE symbol = $1
                LIMIT 1
            "#,
            *symbol as models::ship_module::Symbol
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnectorAsync for ModuleInfo {
    type ID = models::ship_module::Symbol;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &super::DbPool, item: &ModuleInfo) -> crate::Result<Self::ID> {
        Self::upsert(database_pool, item).await?;
        Ok(item.symbol)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &super::DbPool, item: &ModuleInfo) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO module_info (
                    symbol,
                    name,
                    description,
                    range,
                    capacity,
                    power_required,
                    crew_required,
                    slots_required
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (symbol) DO UPDATE
                SET name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    range = EXCLUDED.range,
                    capacity = EXCLUDED.capacity,
                    power_required = EXCLUDED.power_required,
                    crew_required = EXCLUDED.crew_required,
                    slots_required = EXCLUDED.slots_required
            "#,
            item.symbol as models::ship_module::Symbol,
            item.name,
            item.description,
            item.range,
            item.capacity,
            item.power_required,
            item.crew_required,
            item.slots_required
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &super::DbPool, item: &ModuleInfo) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[ModuleInfo]) -> crate::Result<()> {
        let (
            symbols,
            names,
            descriptions,
            ranges,
            capacities,
            power_requireds,
            crew_requireds,
            slots_requireds,
        ): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|m| {
            (
                m.symbol,
                m.name.clone(),
                m.description.clone(),
                m.range,
                m.capacity,
                m.power_required,
                m.crew_required,
                m.slots_required,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO module_info (
                symbol,
                name,
                description,
                range,
                capacity,
                power_required,
                crew_required,
                slots_required
            )
            SELECT * FROM UNNEST(
                $1::ship_module_symbol[],
                $2::character varying[],
                $3::character varying[],
                $4::integer[],
                $5::integer[],
                $6::integer[],
                $7::integer[],
                $8::integer[]
            )
            ON CONFLICT (symbol) DO UPDATE
            SET name = EXCLUDED.name,
                description = EXCLUDED.description,
                range = EXCLUDED.range,
                capacity = EXCLUDED.capacity,
                power_required = EXCLUDED.power_required,
                crew_required = EXCLUDED.crew_required,
                slots_required = EXCLUDED.slots_required
            "#,
            &symbols as &[models::ship_module::Symbol],
            &names,
            &descriptions,
            &ranges as &[Option<i32>],
            &capacities as &[Option<i32>],
            &power_requireds as &[Option<i32>],
            &crew_requireds as &[Option<i32>],
            &slots_requireds as &[Option<i32>]
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ModuleInfo>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ModuleInfo,
                    r#"
                        SELECT
                            symbol as "symbol: models::ship_module::Symbol",
                            name,
                            description,
                            range,
                            capacity,
                            power_required,
                            crew_required,
                            slots_required
                        FROM module_info
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
                    ModuleInfo,
                    r#"
                        SELECT
                            symbol as "symbol: models::ship_module::Symbol",
                            name,
                            description,
                            range,
                            capacity,
                            power_required,
                            crew_required,
                            slots_required
                        FROM module_info
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
                        FROM module_info
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
    async fn get_by_id(
        database_pool: &super::DbPool,
        id: &Self::ID,
    ) -> crate::Result<Option<Self>> {
        let item = sqlx::query_as!(
            ModuleInfo,
            r#"
                SELECT
                    symbol as "symbol: models::ship_module::Symbol",
                    name,
                    description,
                    range,
                    capacity,
                    power_required,
                    crew_required,
                    slots_required
                FROM module_info
                WHERE symbol = $1
                LIMIT 1
            "#,
            *id as models::ship_module::Symbol
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(item)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(
        database_pool: &super::DbPool,
        id: &Self::ID,
    ) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM module_info
                WHERE symbol = $1
            "#,
            *id as models::ship_module::Symbol
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.symbol = id;
    }
}
