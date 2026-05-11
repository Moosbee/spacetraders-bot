use space_traders_client::models;
use tracing::instrument;

use super::{run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult};

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "DBReactorInfo")]
pub struct ReactorInfo {
    pub symbol: models::ship_reactor::Symbol,
    pub name: String,
    pub description: String,
    pub power_output: i32,
    pub power_required: Option<i32>,
    pub crew_required: Option<i32>,
    pub slots_required: Option<i32>,
}

impl From<models::ship_reactor::ShipReactor> for ReactorInfo {
    fn from(value: models::ship_reactor::ShipReactor) -> Self {
        ReactorInfo {
            symbol: value.symbol,
            name: value.name,
            description: value.description,
            power_output: value.power_output,
            power_required: value.requirements.power,
            crew_required: value.requirements.crew,
            slots_required: value.requirements.slots,
        }
    }
}

impl ReactorInfo {
    pub async fn get_by_symbol(
        database_pool: &super::DbPool,
        symbol: &models::ship_reactor::Symbol,
    ) -> crate::Result<ReactorInfo> {
        let erg = sqlx::query_as!(
            ReactorInfo,
            r#"
    SELECT
        symbol as "symbol: models::ship_reactor::Symbol",
        name,
        description,
        power_output,
        power_required,
        crew_required,
        slots_required
    FROM reactor_info
    WHERE symbol = $1
    LIMIT 1
    "#,
            *symbol as models::ship_reactor::Symbol
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnectorAsync for ReactorInfo {
    type ID = models::ship_reactor::Symbol;

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert_new(
        database_pool: &super::DbPool,
        item: &ReactorInfo,
    ) -> crate::Result<Self::ID> {
        Self::upsert(database_pool, item).await?;
        Ok(item.symbol)
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn upsert(database_pool: &super::DbPool, item: &ReactorInfo) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO reactor_info (
                    symbol,
                    name,
                    description,
                    power_output,
                    power_required,
                    crew_required,
                    slots_required
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (symbol) DO UPDATE
                SET name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    power_output = EXCLUDED.power_output,
                    power_required = EXCLUDED.power_required,
                    crew_required = EXCLUDED.crew_required,
                    slots_required = EXCLUDED.slots_required
            "#,
            item.symbol as models::ship_reactor::Symbol,
            item.name,
            item.description,
            item.power_output,
            item.power_required,
            item.crew_required,
            item.slots_required
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn update(database_pool: &super::DbPool, item: &ReactorInfo) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[ReactorInfo],
    ) -> crate::Result<()> {
        let (
            symbols,
            names,
            descriptions,
            power_outputs,
            power_requireds,
            crew_requireds,
            slots_requireds,
        ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
            itertools::multiunzip(items.iter().map(|r| {
                (
                    r.symbol,
                    r.name.clone(),
                    r.description.clone(),
                    r.power_output,
                    r.power_required,
                    r.crew_required,
                    r.slots_required,
                )
            }));

        sqlx::query!(
            r#"
            INSERT INTO reactor_info (
                symbol,
                name,
                description,
                power_output,
                power_required,
                crew_required,
                slots_required
            )
            SELECT * FROM UNNEST(
                $1::ship_reactor_symbol[],
                $2::character varying[],
                $3::character varying[],
                $4::integer[],
                $5::integer[],
                $6::integer[],
                $7::integer[]
            )
            ON CONFLICT (symbol) DO UPDATE
            SET name = EXCLUDED.name,
                description = EXCLUDED.description,
                power_output = EXCLUDED.power_output,
                power_required = EXCLUDED.power_required,
                crew_required = EXCLUDED.crew_required,
                slots_required = EXCLUDED.slots_required
            "#,
            &symbols as &[models::ship_reactor::Symbol],
            &names,
            &descriptions,
            &power_outputs,
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
    ) -> crate::Result<PaginatedResult<ReactorInfo>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ReactorInfo,
                    r#"
                        SELECT
                            symbol as "symbol: models::ship_reactor::Symbol",
                            name,
                            description,
                            power_output,
                            power_required,
                            crew_required,
                            slots_required
                        FROM reactor_info
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
                    ReactorInfo,
                    r#"
                        SELECT
                            symbol as "symbol: models::ship_reactor::Symbol",
                            name,
                            description,
                            power_output,
                            power_required,
                            crew_required,
                            slots_required
                        FROM reactor_info
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
                        FROM reactor_info
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
            ReactorInfo,
            r#"
                SELECT
                    symbol as "symbol: models::ship_reactor::Symbol",
                    name,
                    description,
                    power_output,
                    power_required,
                    crew_required,
                    slots_required
                FROM reactor_info
                WHERE symbol = $1
                LIMIT 1
            "#,
            *id as models::ship_reactor::Symbol
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
                DELETE FROM reactor_info
                WHERE symbol = $1
            "#,
            *id as models::ship_reactor::Symbol
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.symbol = id;
    }
}
