use chrono::{DateTime, Utc};
use space_traders_client::models;

use super::DatabaseConnector;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ShipyardShip {
    #[allow(dead_code)]
    pub id: i64,
    pub waypoint_symbol: String,
    pub ship_type: models::ShipType,
    pub name: String,
    pub supply: models::SupplyLevel,
    pub activity: Option<models::ActivityLevel>,
    pub purchase_price: i32,
    pub frame_type: models::ship_frame::Symbol,
    pub frame_quality: Option<f64>,
    pub reactor_type: models::ship_reactor::Symbol,
    pub reactor_quality: Option<f64>,
    pub engine_type: models::ship_engine::Symbol,
    pub engine_quality: Option<f64>,
    pub modules: Vec<models::ship_module::Symbol>,
    pub mounts: Vec<models::ship_mount::Symbol>,
    pub crew_requirement: i32,
    pub crew_capacity: i32,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl ShipyardShip {
    pub fn with_waypoint(value: models::ShipyardShip, waypoint_symbol: &str) -> Self {
        Self {
            id: 0,
            waypoint_symbol: waypoint_symbol.to_string(),
            ship_type: value.r#type,
            name: value.name,
            supply: value.supply,
            activity: value.activity,
            purchase_price: value.purchase_price,
            frame_type: value.frame.symbol,
            frame_quality: value.frame.quality,
            reactor_type: value.reactor.symbol,
            reactor_quality: value.reactor.quality,
            engine_type: value.engine.symbol,
            engine_quality: value.engine.quality,
            modules: value.modules.iter().map(|m| m.symbol).collect::<Vec<_>>(),
            mounts: value.mounts.iter().map(|m| m.symbol).collect::<Vec<_>>(),
            crew_requirement: value.crew.required,
            crew_capacity: value.crew.capacity,
            created_at: DateTime::<Utc>::MIN_UTC,
        }
    }

    pub async fn get_last_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> sqlx::Result<Vec<ShipyardShip>> {
        sqlx::query_as!(
            ShipyardShip,
            r#"
            SELECT DISTINCT ON (ship_type)
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                name,
                supply as "supply: models::SupplyLevel",
                activity as "activity: models::ActivityLevel",
                purchase_price,
                frame_type as "frame_type: models::ship_frame::Symbol",
                frame_quality,
                reactor_type as "reactor_type: models::ship_reactor::Symbol",
                reactor_quality,
                engine_type as "engine_type: models::ship_engine::Symbol",
                engine_quality,
                modules as "modules: Vec<models::ship_module::Symbol>",
                mounts as "mounts: Vec<models::ship_mount::Symbol>",
                crew_requirement,
                crew_capacity,
                created_at
            FROM shipyard_ship
            WHERE waypoint_symbol = $1
            ORDER BY ship_type, created_at DESC
            "#,
            waypoint_symbol
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl DatabaseConnector<ShipyardShip> for ShipyardShip {
    async fn insert(database_pool: &super::DbPool, item: &ShipyardShip) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO shipyard_ship (
                    waypoint_symbol,
                    ship_type,
                    name,
                    supply,
                    activity,
                    purchase_price,
                    frame_type,
                    frame_quality,
                    reactor_type,
                    reactor_quality,
                    engine_type,
                    engine_quality,
                    modules,
                    mounts,
                    crew_requirement,
                    crew_capacity
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                ON CONFLICT (id) DO UPDATE
                SET waypoint_symbol = EXCLUDED.waypoint_symbol,
                    ship_type = EXCLUDED.ship_type,
                    name = EXCLUDED.name,
                    supply = EXCLUDED.supply,
                    activity = EXCLUDED.activity,
                    purchase_price = EXCLUDED.purchase_price,
                    frame_type = EXCLUDED.frame_type,
                    frame_quality = EXCLUDED.frame_quality,
                    reactor_type = EXCLUDED.reactor_type,
                    reactor_quality = EXCLUDED.reactor_quality,
                    engine_type = EXCLUDED.engine_type,
                    engine_quality = EXCLUDED.engine_quality,
                    modules = EXCLUDED.modules,
                    mounts = EXCLUDED.mounts,
                    crew_requirement = EXCLUDED.crew_requirement,
                    crew_capacity = EXCLUDED.crew_capacity,
                    created_at = now()
            "#,
            item.waypoint_symbol,
            item.ship_type as models::ShipType,
            item.name,
            item.supply as models::SupplyLevel,
            item.activity as Option<models::ActivityLevel>,
            item.purchase_price,
            item.frame_type as models::ship_frame::Symbol,
            item.frame_quality,
            item.reactor_type as models::ship_reactor::Symbol,
            item.reactor_quality,
            item.engine_type as models::ship_engine::Symbol,
            item.engine_quality,
            &item.modules as &[models::ship_module::Symbol],
            &item.mounts as &[models::ship_mount::Symbol],
            item.crew_requirement,
            item.crew_capacity
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[ShipyardShip],
    ) -> sqlx::Result<()> {
        for item in items {
            Self::insert(database_pool, item).await?;
        }

        Ok(())
    }

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<ShipyardShip>> {
        sqlx::query_as!(
            ShipyardShip,
            r#"
            SELECT
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                name,
                supply as "supply: models::SupplyLevel",
                activity as "activity: models::ActivityLevel",
                purchase_price,
                frame_type as "frame_type: models::ship_frame::Symbol",
                frame_quality,
                reactor_type as "reactor_type: models::ship_reactor::Symbol",
                reactor_quality,
                engine_type as "engine_type: models::ship_engine::Symbol",
                engine_quality,
                modules as "modules: Vec<models::ship_module::Symbol>",
                mounts as "mounts: Vec<models::ship_mount::Symbol>",
                crew_requirement,
                crew_capacity,
                created_at
            FROM shipyard_ship
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
