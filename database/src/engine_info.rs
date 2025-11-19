use space_traders_client::models;
use tracing::instrument;

use super::DatabaseConnector;

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "DBEngineInfo")]
pub struct EngineInfo {
    pub symbol: models::ship_engine::Symbol,
    pub name: String,
    pub description: String,
    pub speed: i32,
    pub power_required: Option<i32>,
    pub crew_required: Option<i32>,
    pub slots_required: Option<i32>,
}

impl From<models::ship_engine::ShipEngine> for EngineInfo {
    fn from(value: models::ship_engine::ShipEngine) -> Self {
        EngineInfo {
            symbol: value.symbol,
            name: value.name,
            description: value.description,
            speed: value.speed,
            power_required: value.requirements.power,
            crew_required: value.requirements.crew,
            slots_required: value.requirements.slots,
        }
    }
}

impl EngineInfo {
    pub async fn get_by_symbol(
        database_pool: &super::DbPool,
        symbol: &models::ship_engine::Symbol,
    ) -> crate::Result<EngineInfo> {
        let erg = sqlx::query_as!(
            EngineInfo,
            r#"
        SELECT
            symbol as "symbol: models::ship_engine::Symbol",
            name,
            description,
            speed,
            power_required,
            crew_required,
            slots_required
        FROM engine_info
        WHERE symbol = $1
        LIMIT 1
        "#,
            *symbol as models::ship_engine::Symbol
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnector<EngineInfo> for EngineInfo {
    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert(database_pool: &super::DbPool, item: &EngineInfo) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO engine_info (
                    symbol,
                    name,
                    description,
                    speed,
                    power_required,
                    crew_required,
                    slots_required
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (symbol) DO UPDATE
                SET name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    speed = EXCLUDED.speed,
                    power_required = EXCLUDED.power_required,
                    crew_required = EXCLUDED.crew_required,
                    slots_required = EXCLUDED.slots_required
            "#,
            item.symbol as models::ship_engine::Symbol,
            item.name,
            item.description,
            item.speed,
            item.power_required,
            item.crew_required,
            item.slots_required
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[EngineInfo]) -> crate::Result<()> {
        let (
            symbols,
            names,
            descriptions,
            speeds,
            power_requireds,
            crew_requireds,
            slots_requireds,
        ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
            itertools::multiunzip(items.iter().map(|e| {
                (
                    e.symbol,
                    e.name.clone(),
                    e.description.clone(),
                    e.speed,
                    e.power_required,
                    e.crew_required,
                    e.slots_required,
                )
            }));

        sqlx::query!(
            r#"
            INSERT INTO engine_info (
                symbol,
                name,
                description,
                speed,
                power_required,
                crew_required,
                slots_required
            )
            SELECT * FROM UNNEST(
                $1::ship_engine_symbol[],
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
                speed = EXCLUDED.speed,
                power_required = EXCLUDED.power_required,
                crew_required = EXCLUDED.crew_required,
                slots_required = EXCLUDED.slots_required
            "#,
            &symbols as &[models::ship_engine::Symbol],
            &names,
            &descriptions,
            &speeds,
            &power_requireds as &[Option<i32>],
            &crew_requireds as &[Option<i32>],
            &slots_requireds as &[Option<i32>]
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<EngineInfo>> {
        let erg = sqlx::query_as!(
            EngineInfo,
            r#"
            SELECT
                symbol as "symbol: models::ship_engine::Symbol",
                name,
                description,
                speed,
                power_required,
                crew_required,
                slots_required
            FROM engine_info
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
