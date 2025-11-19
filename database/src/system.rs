use space_traders_client::models;
use tracing::instrument;

use super::DatabaseConnector;

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBSystem")]
pub struct System {
    pub symbol: String,
    pub sector_symbol: String,
    pub system_type: models::SystemType,
    pub x: i32,
    pub y: i32,
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

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct RespSystem {
    pub symbol: String,
    pub sector_symbol: String,
    pub system_type: models::SystemType,
    pub x: i32,
    pub y: i32,
    pub waypoints: Option<i32>,
    pub marketplaces: Option<i32>,
    pub shipyards: Option<i32>,
    pub has_my_ships: Option<bool>,
}

impl From<&models::System> for System {
    fn from(system: &models::System) -> Self {
        System {
            symbol: system.symbol.clone(),
            sector_symbol: system.sector_symbol.clone(),
            system_type: system.r#type,
            x: system.x,
            y: system.y,
        }
    }
}

impl RespSystem {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<RespSystem>> {
        let erg = sqlx::query_as!(
            RespSystem,
            r#"
            SELECT 
                system.symbol,
                system.sector_symbol,
                system.system_type as "system_type: models::SystemType",
                system.x,
                system.y,
            		count(waypoint.symbol) as "waypoints: i32",
				      	sum(CASE when waypoint.has_shipyard THEN 1 ELSE 0 END) as "shipyards: i32",
			      		sum(CASE when waypoint.has_marketplace THEN 1 ELSE 0 END) as "marketplaces: i32",
            		false as "has_my_ships: bool"
            FROM system left join waypoint on system.symbol = waypoint.system_symbol
			group by system.symbol
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;

        Ok(erg)
    }
}

impl System {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_symbol(
        database_pool: &super::DbPool,
        symbol: &str,
    ) -> crate::Result<Option<Self>> {
        let erg = sqlx::query_as!(
            System,
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
            symbol
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnector<System> for System {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &super::DbPool, item: &System) -> crate::Result<()> {
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

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[System]) -> crate::Result<()> {
        let (symbols, sector_symbols, system_types, xs, ys): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
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

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<System>> {
        let erg = sqlx::query_as!(
            System,
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
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl System {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_id(
        database_pool: &super::DbPool,
        id: &String,
    ) -> crate::Result<Option<System>> {
        let erg = sqlx::query_as!(
            System,
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
        .await?;
        Ok(erg)
    }
}
