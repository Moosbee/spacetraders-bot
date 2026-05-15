use std::collections::HashMap;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

// #[derive(sqlx::FromRow)]
type CargoInv = HashMap<models::TradeSymbol, i32>;
type JsonPayload = sqlx::types::Json<serde_json::Value>;

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "DBShipState")]
pub struct ShipState {
    #[allow(dead_code)]
    pub id: i64,
    // Basic ship identification
    pub symbol: String,
    pub display_name: String,

    // Performance
    pub engine_speed: i32,

    // Conditions
    pub engine_condition: f64,
    pub engine_integrity: f64,
    pub frame_condition: f64,
    pub frame_integrity: f64,
    pub reactor_condition: f64,
    pub reactor_integrity: f64,

    // Fuel
    pub fuel_capacity: i32,
    pub fuel_current: i32,

    // Cargo
    pub cargo_capacity: i32,
    pub cargo_units: i32,
    // pub cargo_inventory: HashMap<models::TradeSymbol, i32>,
    #[graphql(skip)]
    pub cargo_inventory: sqlx::types::Json<CargoInv>,
    #[graphql(skip)]
    pub status: JsonPayload,

    // Mounts and Modules
    pub mounts: Vec<models::ship_mount::Symbol>,
    pub modules: Vec<models::ship_module::Symbol>,

    // Reactor, Frame and Engine
    pub reactor_symbol: models::ship_reactor::Symbol,
    pub frame_symbol: models::ship_frame::Symbol,
    pub engine_symbol: models::ship_engine::Symbol,

    // Cooldown
    pub cooldown_expiration: Option<DateTime<Utc>>,
    pub cooldown: Option<i32>,

    // Navigation
    pub flight_mode: String, //models::ShipNavFlightMode,
    pub nav_status: String,  //models::ShipNavStatus,
    pub system_symbol: String,
    pub waypoint_symbol: String,

    // Route
    pub route_arrival: DateTime<Utc>,
    pub route_departure: DateTime<Utc>,
    pub route_destination_symbol: String,
    pub route_destination_system: String,
    pub route_origin_symbol: String,
    pub route_origin_system: String,

    // Auto Pilot
    pub auto_pilot_arrival: Option<DateTime<Utc>>,
    pub auto_pilot_departure_time: Option<DateTime<Utc>>,
    pub auto_pilot_destination_symbol: Option<String>,
    pub auto_pilot_destination_system_symbol: Option<String>,
    pub auto_pilot_origin_symbol: Option<String>,
    pub auto_pilot_origin_system_symbol: Option<String>,
    pub auto_pilot_distance: Option<f64>,
    pub auto_pilot_fuel_cost: Option<i32>,
    pub auto_pilot_travel_time: Option<f64>,
    #[graphql(skip)]
    pub auto_pilot_state: JsonPayload,

    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl ShipState {
    #[instrument(level = "trace", skip(database_pool, item))]
    pub async fn insert_get_id(database_pool: &DbPool, item: &ShipState) -> crate::Result<i64> {
        let id = sqlx::query!(
            r#"
                INSERT INTO ship_state (
                  symbol,
                  display_name,
                  engine_speed,
                  engine_condition,
                  engine_integrity,
                  frame_condition,
                  frame_integrity,
                  reactor_condition,
                  reactor_integrity,
                  fuel_capacity,
                  fuel_current,
                  cargo_capacity,
                  cargo_units,
                  cargo_inventory,
                  status,
                  mounts,
                  modules,
                  cooldown_expiration,
                  cooldown,
                  reactor_symbol,
                  frame_symbol,
                  engine_symbol,
                  flight_mode,
                  nav_status,
                  system_symbol,
                  waypoint_symbol,
                  route_arrival,
                  route_departure,
                  route_destination_symbol,
                  route_destination_system,
                  route_origin_symbol,
                  route_origin_system,
                  auto_pilot_arrival,
                  auto_pilot_departure_time,
                  auto_pilot_destination_symbol,
                  auto_pilot_destination_system_symbol,
                  auto_pilot_origin_symbol,
                  auto_pilot_origin_system_symbol,
                  auto_pilot_distance,
                  auto_pilot_fuel_cost,
                                    auto_pilot_travel_time,
                                    auto_pilot_state
                )
                VALUES (
                  $1,
                  $2,
                  $3,
                  $4,
                  $5,
                  $6,
                  $7,
                  $8,
                  $9,
                  $10,
                  $11,
                  $12,
                  $13,
                  $14::jsonb,
                  $15::jsonb,
                  $16::ship_mount_symbol[],
                  $17::ship_module_symbol[],
                  $18,
                  $19,
                  $20::ship_reactor_symbol,
                  $21::ship_frame_symbol,
                  $22::ship_engine_symbol,
                  $23,
                  $24,
                  $25,
                  $26,
                  $27,
                  $28,
                  $29,
                  $30,
                  $31,
                  $32,
                  $33,
                  $34,
                  $35,
                  $36,
                  $37,
                  $38,
                  $39,
                  $40,
                  $41,
                  $42::jsonb
                )
                RETURNING id;
            "#,
            &item.symbol,
            &item.display_name,
            &item.engine_speed,
            &item.engine_condition,
            &item.engine_integrity,
            &item.frame_condition,
            &item.frame_integrity,
            &item.reactor_condition,
            &item.reactor_integrity,
            &item.fuel_capacity,
            &item.fuel_current,
            &item.cargo_capacity,
            &item.cargo_units,
            &item.cargo_inventory as &sqlx::types::Json<HashMap<models::TradeSymbol, i32>>,
            &item.status as &JsonPayload,
            &item.mounts as &[models::ship_mount::Symbol],
            &item.modules as &[models::ship_module::Symbol],
            &item.cooldown_expiration as &Option<DateTime<Utc>>,
            &item.cooldown as &Option<i32>,
            &item.reactor_symbol as &models::ship_reactor::Symbol,
            &item.frame_symbol as &models::ship_frame::Symbol,
            &item.engine_symbol as &models::ship_engine::Symbol,
            &item.flight_mode,
            &item.nav_status,
            &item.system_symbol,
            &item.waypoint_symbol,
            &item.route_arrival,
            &item.route_departure,
            &item.route_destination_symbol,
            &item.route_destination_system,
            &item.route_origin_symbol,
            &item.route_origin_system,
            &item.auto_pilot_arrival as &Option<DateTime<Utc>>,
            &item.auto_pilot_departure_time as &Option<DateTime<Utc>>,
            &item.auto_pilot_destination_symbol as &Option<String>,
            &item.auto_pilot_destination_system_symbol as &Option<String>,
            &item.auto_pilot_origin_symbol as &Option<String>,
            &item.auto_pilot_origin_system_symbol as &Option<String>,
            &item.auto_pilot_distance as &Option<f64>,
            &item.auto_pilot_fuel_cost as &Option<i32>,
            &item.auto_pilot_travel_time as &Option<f64>,
            &item.auto_pilot_state as &JsonPayload,
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        Ok(id.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_waypoint(
        database_pool: &DbPool,
        waypoint: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipState>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipState,
                    r#"
                        SELECT
                          id,
                          symbol,
                          display_name,
                          engine_speed,
                          engine_condition,
                          engine_integrity,
                          frame_condition,
                          frame_integrity,
                          reactor_condition,
                          reactor_integrity,
                          fuel_capacity,
                          fuel_current,
                          cargo_capacity,
                          cargo_units,
                          cargo_inventory as "cargo_inventory: sqlx::types::Json<CargoInv>",
                          status as "status: JsonPayload",
                          mounts as "mounts: Vec<models::ship_mount::Symbol>",
                          modules as "modules: Vec<models::ship_module::Symbol>",
                          reactor_symbol as "reactor_symbol: models::ship_reactor::Symbol",
                          frame_symbol as "frame_symbol: models::ship_frame::Symbol",
                          engine_symbol as "engine_symbol: models::ship_engine::Symbol",
                          cooldown_expiration,
                          cooldown,
                          flight_mode,
                          nav_status,
                          system_symbol,
                          waypoint_symbol,
                          route_arrival,
                          route_departure,
                          route_destination_symbol,
                          route_destination_system,
                          route_origin_symbol,
                          route_origin_system,
                          auto_pilot_arrival,
                          auto_pilot_departure_time,
                          auto_pilot_destination_symbol,
                          auto_pilot_destination_system_symbol,
                          auto_pilot_origin_symbol,
                          auto_pilot_origin_system_symbol,
                          auto_pilot_distance,
                          auto_pilot_fuel_cost,
                          auto_pilot_travel_time,
                                                    auto_pilot_state as "auto_pilot_state: JsonPayload",
                          created_at
                        FROM ship_state
                        WHERE waypoint_symbol = $1
                        ORDER BY created_at
                        LIMIT $2 OFFSET $3
                    "#,
                    waypoint,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ShipState,
                    r#"
                        SELECT
                          id,
                          symbol,
                          display_name,
                          engine_speed,
                          engine_condition,
                          engine_integrity,
                          frame_condition,
                          frame_integrity,
                          reactor_condition,
                          reactor_integrity,
                          fuel_capacity,
                          fuel_current,
                          cargo_capacity,
                          cargo_units,
                          cargo_inventory as "cargo_inventory: sqlx::types::Json<CargoInv>",
                          status as "status: JsonPayload",
                          mounts as "mounts: Vec<models::ship_mount::Symbol>",
                          modules as "modules: Vec<models::ship_module::Symbol>",
                          reactor_symbol as "reactor_symbol: models::ship_reactor::Symbol",
                          frame_symbol as "frame_symbol: models::ship_frame::Symbol",
                          engine_symbol as "engine_symbol: models::ship_engine::Symbol",
                          cooldown_expiration,
                          cooldown,
                          flight_mode,
                          nav_status,
                          system_symbol,
                          waypoint_symbol,
                          route_arrival,
                          route_departure,
                          route_destination_symbol,
                          route_destination_system,
                          route_origin_symbol,
                          route_origin_system,
                          auto_pilot_arrival,
                          auto_pilot_departure_time,
                          auto_pilot_destination_symbol,
                          auto_pilot_destination_system_symbol,
                          auto_pilot_origin_symbol,
                          auto_pilot_origin_system_symbol,
                          auto_pilot_distance,
                          auto_pilot_fuel_cost,
                          auto_pilot_travel_time,
                                                    auto_pilot_state as "auto_pilot_state: JsonPayload",
                          created_at
                        FROM ship_state
                        WHERE waypoint_symbol = $1
                        ORDER BY created_at
                    "#,
                    waypoint
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ship_state
                    WHERE waypoint_symbol = $1
                    "#,
                    waypoint
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_system(
        database_pool: &DbPool,
        system: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipState>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipState,
                    r#"
                        SELECT
                          id,
                          symbol,
                          display_name,
                          engine_speed,
                          engine_condition,
                          engine_integrity,
                          frame_condition,
                          frame_integrity,
                          reactor_condition,
                          reactor_integrity,
                          fuel_capacity,
                          fuel_current,
                          cargo_capacity,
                          cargo_units,
                          cargo_inventory as "cargo_inventory: sqlx::types::Json<CargoInv>",
                          status as "status: JsonPayload",
                          mounts as "mounts: Vec<models::ship_mount::Symbol>",
                          modules as "modules: Vec<models::ship_module::Symbol>",
                          reactor_symbol as "reactor_symbol: models::ship_reactor::Symbol",
                          frame_symbol as "frame_symbol: models::ship_frame::Symbol",
                          engine_symbol as "engine_symbol: models::ship_engine::Symbol",
                          cooldown_expiration,
                          cooldown,
                          flight_mode,
                          nav_status,
                          system_symbol,
                          waypoint_symbol,
                          route_arrival,
                          route_departure,
                          route_destination_symbol,
                          route_destination_system,
                          route_origin_symbol,
                          route_origin_system,
                          auto_pilot_arrival,
                          auto_pilot_departure_time,
                          auto_pilot_destination_symbol,
                          auto_pilot_destination_system_symbol,
                          auto_pilot_origin_symbol,
                          auto_pilot_origin_system_symbol,
                          auto_pilot_distance,
                          auto_pilot_fuel_cost,
                          auto_pilot_travel_time,
                                                    auto_pilot_state as "auto_pilot_state: JsonPayload",
                          created_at
                        FROM ship_state
                        WHERE system_symbol = $1
                        ORDER BY created_at
                        LIMIT $2 OFFSET $3
                    "#,
                    system,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ShipState,
                    r#"
                        SELECT
                          id,
                          symbol,
                          display_name,
                          engine_speed,
                          engine_condition,
                          engine_integrity,
                          frame_condition,
                          frame_integrity,
                          reactor_condition,
                          reactor_integrity,
                          fuel_capacity,
                          fuel_current,
                          cargo_capacity,
                          cargo_units,
                          cargo_inventory as "cargo_inventory: sqlx::types::Json<CargoInv>",
                          status as "status: JsonPayload",
                          mounts as "mounts: Vec<models::ship_mount::Symbol>",
                          modules as "modules: Vec<models::ship_module::Symbol>",
                          reactor_symbol as "reactor_symbol: models::ship_reactor::Symbol",
                          frame_symbol as "frame_symbol: models::ship_frame::Symbol",
                          engine_symbol as "engine_symbol: models::ship_engine::Symbol",
                          cooldown_expiration,
                          cooldown,
                          flight_mode,
                          nav_status,
                          system_symbol,
                          waypoint_symbol,
                          route_arrival,
                          route_departure,
                          route_destination_symbol,
                          route_destination_system,
                          route_origin_symbol,
                          route_origin_system,
                          auto_pilot_arrival,
                          auto_pilot_departure_time,
                          auto_pilot_destination_symbol,
                          auto_pilot_destination_system_symbol,
                          auto_pilot_origin_symbol,
                          auto_pilot_origin_system_symbol,
                          auto_pilot_distance,
                          auto_pilot_fuel_cost,
                          auto_pilot_travel_time,
                                                    auto_pilot_state as "auto_pilot_state: JsonPayload",
                          created_at
                        FROM ship_state
                        WHERE system_symbol = $1
                        ORDER BY created_at
                    "#,
                    system
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ship_state
                    WHERE system_symbol = $1
                    "#,
                    system
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_ship(
        database_pool: &DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipState>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipState,
                    r#"
                        SELECT
                          id,
                          symbol,
                          display_name,
                          engine_speed,
                          engine_condition,
                          engine_integrity,
                          frame_condition,
                          frame_integrity,
                          reactor_condition,
                          reactor_integrity,
                          fuel_capacity,
                          fuel_current,
                          cargo_capacity,
                          cargo_units,
                          cargo_inventory as "cargo_inventory: sqlx::types::Json<CargoInv>",
                          status as "status: JsonPayload",
                          mounts as "mounts: Vec<models::ship_mount::Symbol>",
                          modules as "modules: Vec<models::ship_module::Symbol>",
                          reactor_symbol as "reactor_symbol: models::ship_reactor::Symbol",
                          frame_symbol as "frame_symbol: models::ship_frame::Symbol",
                          engine_symbol as "engine_symbol: models::ship_engine::Symbol",
                          cooldown_expiration,
                          cooldown,
                          flight_mode,
                          nav_status,
                          system_symbol,
                          waypoint_symbol,
                          route_arrival,
                          route_departure,
                          route_destination_symbol,
                          route_destination_system,
                          route_origin_symbol,
                          route_origin_system,
                          auto_pilot_arrival,
                          auto_pilot_departure_time,
                          auto_pilot_destination_symbol,
                          auto_pilot_destination_system_symbol,
                          auto_pilot_origin_symbol,
                          auto_pilot_origin_system_symbol,
                          auto_pilot_distance,
                          auto_pilot_fuel_cost,
                          auto_pilot_travel_time,
                                                    auto_pilot_state as "auto_pilot_state: JsonPayload",
                          created_at
                        FROM ship_state
                        WHERE symbol = $1
                        ORDER BY created_at
                        LIMIT $2 OFFSET $3
                    "#,
                    ship_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ShipState,
                    r#"
                        SELECT
                          id,
                          symbol,
                          display_name,
                          engine_speed,
                          engine_condition,
                          engine_integrity,
                          frame_condition,
                          frame_integrity,
                          reactor_condition,
                          reactor_integrity,
                          fuel_capacity,
                          fuel_current,
                          cargo_capacity,
                          cargo_units,
                          cargo_inventory as "cargo_inventory: sqlx::types::Json<CargoInv>",
                          status as "status: JsonPayload",
                          mounts as "mounts: Vec<models::ship_mount::Symbol>",
                          modules as "modules: Vec<models::ship_module::Symbol>",
                          reactor_symbol as "reactor_symbol: models::ship_reactor::Symbol",
                          frame_symbol as "frame_symbol: models::ship_frame::Symbol",
                          engine_symbol as "engine_symbol: models::ship_engine::Symbol",
                          cooldown_expiration,
                          cooldown,
                          flight_mode,
                          nav_status,
                          system_symbol,
                          waypoint_symbol,
                          route_arrival,
                          route_departure,
                          route_destination_symbol,
                          route_destination_system,
                          route_origin_symbol,
                          route_origin_system,
                          auto_pilot_arrival,
                          auto_pilot_departure_time,
                          auto_pilot_destination_symbol,
                          auto_pilot_destination_system_symbol,
                          auto_pilot_origin_symbol,
                          auto_pilot_origin_system_symbol,
                          auto_pilot_distance,
                          auto_pilot_fuel_cost,
                          auto_pilot_travel_time,
                                                    auto_pilot_state as "auto_pilot_state: JsonPayload",
                          created_at
                        FROM ship_state
                        WHERE symbol = $1
                        ORDER BY created_at
                    "#,
                    ship_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM ship_state
                    WHERE symbol = $1
                    "#,
                    ship_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }
}

impl DatabaseConnectorAsync for ShipState {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert_new(database_pool: &DbPool, item: &ShipState) -> crate::Result<Self::ID> {
        Self::insert_get_id(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn upsert(database_pool: &DbPool, item: &ShipState) -> crate::Result<()> {
        let _id = Self::insert_get_id(database_pool, item).await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, item))]
    async fn update(database_pool: &DbPool, item: &ShipState) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[ShipState]) -> crate::Result<()> {
        for item in items {
            Self::upsert(database_pool, item).await?;
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ShipState>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ShipState,
                    r#"
                        SELECT
                          id,
                          symbol,
                          display_name,
                          engine_speed,
                          engine_condition,
                          engine_integrity,
                          frame_condition,
                          frame_integrity,
                          reactor_condition,
                          reactor_integrity,
                          fuel_capacity,
                          fuel_current,
                          cargo_capacity,
                          cargo_units,
                          cargo_inventory as "cargo_inventory: sqlx::types::Json<CargoInv>",
                          status as "status: JsonPayload",
                          mounts as "mounts: Vec<models::ship_mount::Symbol>",
                          modules as "modules: Vec<models::ship_module::Symbol>",
                          reactor_symbol as "reactor_symbol: models::ship_reactor::Symbol",
                          frame_symbol as "frame_symbol: models::ship_frame::Symbol",
                          engine_symbol as "engine_symbol: models::ship_engine::Symbol",
                          cooldown_expiration,
                          cooldown,
                          flight_mode,
                          nav_status,
                          system_symbol,
                          waypoint_symbol,
                          route_arrival,
                          route_departure,
                          route_destination_symbol,
                          route_destination_system,
                          route_origin_symbol,
                          route_origin_system,
                          auto_pilot_arrival,
                          auto_pilot_departure_time,
                          auto_pilot_destination_symbol,
                          auto_pilot_destination_system_symbol,
                          auto_pilot_origin_symbol,
                          auto_pilot_origin_system_symbol,
                          auto_pilot_distance,
                          auto_pilot_fuel_cost,
                          auto_pilot_travel_time,
                                                    auto_pilot_state as "auto_pilot_state: JsonPayload",
                          created_at
                        FROM ship_state
                        ORDER BY created_at
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
                    ShipState,
                    r#"
                        SELECT
                          id,
                          symbol,
                          display_name,
                          engine_speed,
                          engine_condition,
                          engine_integrity,
                          frame_condition,
                          frame_integrity,
                          reactor_condition,
                          reactor_integrity,
                          fuel_capacity,
                          fuel_current,
                          cargo_capacity,
                          cargo_units,
                          cargo_inventory as "cargo_inventory: sqlx::types::Json<CargoInv>",
                          status as "status: JsonPayload",
                          mounts as "mounts: Vec<models::ship_mount::Symbol>",
                          modules as "modules: Vec<models::ship_module::Symbol>",
                          reactor_symbol as "reactor_symbol: models::ship_reactor::Symbol",
                          frame_symbol as "frame_symbol: models::ship_frame::Symbol",
                          engine_symbol as "engine_symbol: models::ship_engine::Symbol",
                          cooldown_expiration,
                          cooldown,
                          flight_mode,
                          nav_status,
                          system_symbol,
                          waypoint_symbol,
                          route_arrival,
                          route_departure,
                          route_destination_symbol,
                          route_destination_system,
                          route_origin_symbol,
                          route_origin_system,
                          auto_pilot_arrival,
                          auto_pilot_departure_time,
                          auto_pilot_destination_symbol,
                          auto_pilot_destination_system_symbol,
                          auto_pilot_origin_symbol,
                          auto_pilot_origin_system_symbol,
                          auto_pilot_distance,
                          auto_pilot_fuel_cost,
                          auto_pilot_travel_time,
                                                    auto_pilot_state as "auto_pilot_state: JsonPayload",
                          created_at
                        FROM ship_state
                        ORDER BY created_at
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
                    FROM ship_state
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
            ShipState,
            r#"
                SELECT
                  id,
                  symbol,
                  display_name,
                  engine_speed,
                  engine_condition,
                  engine_integrity,
                  frame_condition,
                  frame_integrity,
                  reactor_condition,
                  reactor_integrity,
                  fuel_capacity,
                  fuel_current,
                  cargo_capacity,
                  cargo_units,
                  cargo_inventory as "cargo_inventory: sqlx::types::Json<CargoInv>",
                  status as "status: JsonPayload",
                  mounts as "mounts: Vec<models::ship_mount::Symbol>",
                  modules as "modules: Vec<models::ship_module::Symbol>",
                  reactor_symbol as "reactor_symbol: models::ship_reactor::Symbol",
                  frame_symbol as "frame_symbol: models::ship_frame::Symbol",
                  engine_symbol as "engine_symbol: models::ship_engine::Symbol",
                  cooldown_expiration,
                  cooldown,
                  flight_mode,
                  nav_status,
                  system_symbol,
                  waypoint_symbol,
                  route_arrival,
                  route_departure,
                  route_destination_symbol,
                  route_destination_system,
                  route_origin_symbol,
                  route_origin_system,
                  auto_pilot_arrival,
                  auto_pilot_departure_time,
                  auto_pilot_destination_symbol,
                  auto_pilot_destination_system_symbol,
                  auto_pilot_origin_symbol,
                  auto_pilot_origin_system_symbol,
                  auto_pilot_distance,
                  auto_pilot_fuel_cost,
                  auto_pilot_travel_time,
                                    auto_pilot_state as "auto_pilot_state: JsonPayload",
                  created_at
                FROM ship_state
                WHERE id = $1
                LIMIT 1
            "#,
            *id
        )
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM ship_state
            WHERE id = $1
            "#,
            id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = id;
    }
}
