use space_traders_client::models;
use tracing::instrument;

use super::{run_paginated_query, DatabaseConnectorAsync, PaginatedQuery, PaginatedResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash, async_graphql::SimpleObject)]
#[graphql(name = "DBMountInfo")]
pub struct MountInfo {
    pub symbol: models::ship_mount::Symbol,
    pub name: String,
    pub description: String,
    pub strength: Option<i32>,
    pub deposits: Option<Vec<models::TradeSymbol>>,
    pub power_required: Option<i32>,
    pub crew_required: Option<i32>,
    pub slots_required: Option<i32>,
}

impl MountInfo {
    pub async fn get_by_id(
        database_pool: &super::DbPool,
        symbol: &models::ship_mount::Symbol,
    ) -> crate::Result<MountInfo> {
        let erg = sqlx::query_as!(
            MountInfo,
            r#"
            SELECT
                symbol as "symbol: models::ship_mount::Symbol",
                name,
                description,
                strength,
                deposits as "deposits: Vec<models::TradeSymbol>",
                power_required,
                crew_required,
                slots_required
            FROM mount_info
            WHERE symbol = $1
            LIMIT 1
        "#,
            *symbol as models::ship_mount::Symbol
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl From<models::ship_mount::ShipMount> for MountInfo {
    fn from(value: models::ship_mount::ShipMount) -> Self {
        MountInfo {
            symbol: value.symbol,
            name: value.name,
            description: value.description,
            strength: value.strength,
            deposits: value
                .deposits
                .map(|d| d.into_iter().map(|d| d.into()).collect()),
            power_required: value.requirements.power,
            crew_required: value.requirements.crew,
            slots_required: value.requirements.slots,
        }
    }
}

impl DatabaseConnectorAsync for MountInfo {
    type ID = models::ship_mount::Symbol;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &super::DbPool, item: &MountInfo) -> crate::Result<Self::ID> {
        Self::upsert(database_pool, item).await?;
        Ok(item.symbol)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &super::DbPool, item: &MountInfo) -> crate::Result<()> {
        let deposits = item.deposits.clone();
        sqlx::query!(
            r#"
                INSERT INTO mount_info (
                    symbol,
                    name,
                    description,
                    strength,
                    deposits,
                    power_required,
                    crew_required,
                    slots_required
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (symbol) DO UPDATE
                SET name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    strength = EXCLUDED.strength,
                    deposits = EXCLUDED.deposits,
                    power_required = EXCLUDED.power_required,
                    crew_required = EXCLUDED.crew_required,
                    slots_required = EXCLUDED.slots_required
            "#,
            item.symbol as models::ship_mount::Symbol,
            item.name,
            item.description,
            item.strength,
            deposits as Option<Vec<models::TradeSymbol>>,
            item.power_required,
            item.crew_required,
            item.slots_required
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &super::DbPool, item: &MountInfo) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[MountInfo]) -> crate::Result<()> {
        for item in items {
            Self::upsert(database_pool, item).await?;
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &super::DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MountInfo>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MountInfo,
                    r#"
                        SELECT
                            symbol as "symbol: models::ship_mount::Symbol",
                            name,
                            description,
                            strength,
                            deposits as "deposits: Vec<models::TradeSymbol>",
                            power_required,
                            crew_required,
                            slots_required
                        FROM mount_info
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
                    MountInfo,
                    r#"
                        SELECT
                            symbol as "symbol: models::ship_mount::Symbol",
                            name,
                            description,
                            strength,
                            deposits as "deposits: Vec<models::TradeSymbol>",
                            power_required,
                            crew_required,
                            slots_required
                        FROM mount_info
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
                        FROM mount_info
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
            MountInfo,
            r#"
                SELECT
                    symbol as "symbol: models::ship_mount::Symbol",
                    name,
                    description,
                    strength,
                    deposits as "deposits: Vec<models::TradeSymbol>",
                    power_required,
                    crew_required,
                    slots_required
                FROM mount_info
                WHERE symbol = $1
                LIMIT 1
            "#,
            *id as models::ship_mount::Symbol
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
                DELETE FROM mount_info
                WHERE symbol = $1
            "#,
            *id as models::ship_mount::Symbol
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.symbol = id;
    }
}
