use space_traders_client::models;
use tracing::instrument;

use super::{run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult};

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "DBFrameInfo")]
pub struct FrameInfo {
    pub symbol: models::ship_frame::Symbol,
    pub name: String,
    pub description: String,
    pub module_slots: i32,
    pub mounting_points: i32,
    pub fuel_capacity: i32,
    pub power_required: Option<i32>,
    pub crew_required: Option<i32>,
    pub slots_required: Option<i32>,
}

impl From<models::ship_frame::ShipFrame> for FrameInfo {
    fn from(frame: models::ship_frame::ShipFrame) -> Self {
        Self {
            symbol: frame.symbol,
            name: frame.name,
            description: frame.description,
            module_slots: frame.module_slots,
            mounting_points: frame.mounting_points,
            fuel_capacity: frame.fuel_capacity,
            power_required: frame.requirements.power,
            crew_required: frame.requirements.crew,
            slots_required: frame.requirements.slots,
        }
    }
}

impl FrameInfo {
    pub async fn get_by_symbol(
        database_pool: &super::DbPool,
        symbol: &models::ship_frame::Symbol,
    ) -> crate::Result<FrameInfo> {
        let erg = sqlx::query_as!(
            FrameInfo,
            r#"
        SELECT
            symbol as "symbol: models::ship_frame::Symbol",
            name,
            description,
            module_slots,
            mounting_points,
            fuel_capacity,
            power_required,
            crew_required,
            slots_required
        FROM frame_info
        WHERE symbol = $1
        LIMIT 1
        "#,
            *symbol as models::ship_frame::Symbol
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnectorAsync for FrameInfo {
    type ID = models::ship_frame::Symbol;

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert_new(database_pool: &super::DbPool, item: &FrameInfo) -> crate::Result<Self::ID> {
        Self::upsert(database_pool, item).await?;
        Ok(item.symbol)
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn upsert(database_pool: &super::DbPool, item: &FrameInfo) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO frame_info (
                    symbol,
                    name,
                    description,
                    module_slots,
                    mounting_points,
                    fuel_capacity,
                    power_required,
                    crew_required,
                    slots_required
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (symbol) DO UPDATE
                SET name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    module_slots = EXCLUDED.module_slots,
                    mounting_points = EXCLUDED.mounting_points,
                    fuel_capacity = EXCLUDED.fuel_capacity,
                    power_required = EXCLUDED.power_required,
                    crew_required = EXCLUDED.crew_required,
                    slots_required = EXCLUDED.slots_required
            "#,
            item.symbol as models::ship_frame::Symbol,
            item.name,
            item.description,
            item.module_slots,
            item.mounting_points,
            item.fuel_capacity,
            item.power_required,
            item.crew_required,
            item.slots_required
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn update(database_pool: &super::DbPool, item: &FrameInfo) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[FrameInfo]) -> crate::Result<()> {
        let (
            symbols,
            names,
            descriptions,
            module_slots,
            mounting_points,
            fuel_capacities,
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
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|f| {
            (
                f.symbol,
                f.name.clone(),
                f.description.clone(),
                f.module_slots,
                f.mounting_points,
                f.fuel_capacity,
                f.power_required,
                f.crew_required,
                f.slots_required,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO frame_info (
                symbol,
                name,
                description,
                module_slots,
                mounting_points,
                fuel_capacity,
                power_required,
                crew_required,
                slots_required
            )
            SELECT * FROM UNNEST(
                $1::ship_frame_symbol[],
                $2::character varying[],
                $3::character varying[],
                $4::integer[],
                $5::integer[],
                $6::integer[],
                $7::integer[],
                $8::integer[],
                $9::integer[]
            )
            ON CONFLICT (symbol) DO UPDATE
            SET name = EXCLUDED.name,
                description = EXCLUDED.description,
                module_slots = EXCLUDED.module_slots,
                mounting_points = EXCLUDED.mounting_points,
                fuel_capacity = EXCLUDED.fuel_capacity,
                power_required = EXCLUDED.power_required,
                crew_required = EXCLUDED.crew_required,
                slots_required = EXCLUDED.slots_required
            "#,
            &symbols as &[models::ship_frame::Symbol],
            &names,
            &descriptions,
            &module_slots,
            &mounting_points,
            &fuel_capacities,
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
    ) -> crate::Result<PaginatedResult<FrameInfo>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    FrameInfo,
                    r#"
                        SELECT
                            symbol as "symbol: models::ship_frame::Symbol",
                            name,
                            description,
                            module_slots,
                            mounting_points,
                            fuel_capacity,
                            power_required,
                            crew_required,
                            slots_required
                        FROM frame_info
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
                    FrameInfo,
                    r#"
                        SELECT
                            symbol as "symbol: models::ship_frame::Symbol",
                            name,
                            description,
                            module_slots,
                            mounting_points,
                            fuel_capacity,
                            power_required,
                            crew_required,
                            slots_required
                        FROM frame_info
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
                        FROM frame_info
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
            FrameInfo,
            r#"
                SELECT
                    symbol as "symbol: models::ship_frame::Symbol",
                    name,
                    description,
                    module_slots,
                    mounting_points,
                    fuel_capacity,
                    power_required,
                    crew_required,
                    slots_required
                FROM frame_info
                WHERE symbol = $1
                LIMIT 1
            "#,
            *id as models::ship_frame::Symbol
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
                DELETE FROM frame_info
                WHERE symbol = $1
            "#,
            *id as models::ship_frame::Symbol
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.symbol = id;
    }
}
