use space_traders_client::models;
use tracing::instrument;

use super::{
    DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, ShipmentStatus,
    run_paginated_query,
};

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBConstructionShipment")]
pub struct ConstructionShipment {
    pub id: i64,
    pub material_id: i64,
    pub construction_site_waypoint: String,
    pub ship_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub units: i32,
    #[graphql(name = "purchaseSiteWaypoint")]
    pub purchase_waypoint: String,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub updated_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub status: ShipmentStatus,
    pub reserved_fund: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConstructionShipmentSummary {
    pub id: i64,
    pub material_id: i64,
    pub construction_site_waypoint: String,
    pub ship_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub units: i32,
    pub purchase_waypoint: String,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub updated_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub status: ShipmentStatus,
    pub sum: Option<i32>,
    pub expenses: Option<i32>,
    pub income: Option<i32>,
    pub reserved_fund: Option<i64>,
}

impl ConstructionShipment {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_all_in_transit(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE status = 'IN_TRANSIT'
                        ORDER BY created_at DESC, id DESC
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
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE status = 'IN_TRANSIT'
                        ORDER BY created_at DESC, id DESC
                    "#,
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM construction_shipment
                        WHERE status = 'IN_TRANSIT'
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
    pub async fn get_by_destination_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE construction_site_waypoint = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    waypoint_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE construction_site_waypoint = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    waypoint_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM construction_shipment
                        WHERE construction_site_waypoint = $1
                    "#,
                    waypoint_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_source_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE purchase_waypoint = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    waypoint_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE purchase_waypoint = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    waypoint_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM construction_shipment
                        WHERE purchase_waypoint = $1
                    "#,
                    waypoint_symbol
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
        system_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          construction_shipment.created_at,
                          construction_shipment.updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment JOIN waypoint ON construction_shipment.construction_site_waypoint = waypoint.symbol
                        WHERE waypoint.system_symbol = $1
                        ORDER BY construction_shipment.created_at DESC, construction_shipment.id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    system_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          construction_shipment.created_at,
                          construction_shipment.updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment JOIN waypoint ON construction_shipment.construction_site_waypoint = waypoint.symbol
                        WHERE waypoint.system_symbol = $1
                        ORDER BY construction_shipment.created_at DESC, construction_shipment.id DESC
                    "#,
                    system_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM construction_shipment JOIN waypoint ON construction_shipment.construction_site_waypoint = waypoint.symbol
                        WHERE waypoint.system_symbol = $1
                    "#,
                    system_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_trade_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE trade_symbol = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    trade_symbol as &models::TradeSymbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE trade_symbol = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    trade_symbol as &models::TradeSymbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM construction_shipment
                        WHERE trade_symbol = $1
                    "#,
                    trade_symbol as &models::TradeSymbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_material_id(
        database_pool: &DbPool,
        material_id: i64,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE material_id = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    material_id,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE material_id = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    material_id
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM construction_shipment
                        WHERE material_id = $1
                    "#,
                    material_id
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
    ) -> crate::Result<PaginatedResult<ConstructionShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE ship_symbol = $1
                        ORDER BY created_at DESC, id DESC
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
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE ship_symbol = $1
                        ORDER BY created_at DESC, id DESC
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
                        FROM construction_shipment
                        WHERE ship_symbol = $1
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

    pub async fn get_by_reservation_id(
        database_pool: &DbPool,
        id: i64,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE reserved_fund = $1
                        ORDER BY created_at DESC, id DESC
                        LIMIT $2 OFFSET $3
                    "#,
                    id,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        WHERE reserved_fund = $1
                        ORDER BY created_at DESC, id DESC
                    "#,
                    id
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM construction_shipment
                        WHERE reserved_fund = $1
                    "#,
                    id
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_summary(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionShipmentSummary>> {
        run_paginated_query(
                        query,
                        |page_size, offset| async move {
                                let items = sqlx::query_as!(
                                        ConstructionShipmentSummary,
                                        r#"
                                                SELECT
                                                    construction_shipment.id,
                                                    material_id,
                                                    construction_site_waypoint,
                                                    construction_shipment.ship_symbol,
                                                    construction_shipment.trade_symbol as "trade_symbol: models::TradeSymbol",
                                                    construction_shipment.units,
                                                    purchase_waypoint,
                                                    construction_shipment.created_at,
                                                    construction_shipment.updated_at,
                                                    status as "status: ShipmentStatus",
                                                    sum(market_transaction.total_price) as "sum: i32",
                                                    sum(
                                                        CASE
                                                            WHEN market_transaction.type = 'PURCHASE' THEN market_transaction.total_price
                                                            ELSE 0
                                                        END
                                                    ) as "expenses: i32",
                                                    sum(
                                                        CASE
                                                            WHEN market_transaction.type = 'PURCHASE' THEN 0
                                                            ELSE market_transaction.total_price
                                                        END
                                                    ) as "income: i32",
                                                    construction_shipment.reserved_fund
                                                FROM 
                                                    public.construction_shipment 
                                                    left join public.market_transaction ON market_transaction.construction = construction_shipment.id
                                                group by construction_shipment.id
                                                ORDER BY construction_shipment.id ASC
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
                                        ConstructionShipmentSummary,
                                        r#"
                                                SELECT
                                                    construction_shipment.id,
                                                    material_id,
                                                    construction_site_waypoint,
                                                    construction_shipment.ship_symbol,
                                                    construction_shipment.trade_symbol as "trade_symbol: models::TradeSymbol",
                                                    construction_shipment.units,
                                                    purchase_waypoint,
                                                    construction_shipment.created_at,
                                                    construction_shipment.updated_at,
                                                    status as "status: ShipmentStatus",
                                                    sum(market_transaction.total_price) as "sum: i32",
                                                    sum(
                                                        CASE
                                                            WHEN market_transaction.type = 'PURCHASE' THEN market_transaction.total_price
                                                            ELSE 0
                                                        END
                                                    ) as "expenses: i32",
                                                    sum(
                                                        CASE
                                                            WHEN market_transaction.type = 'PURCHASE' THEN 0
                                                            ELSE market_transaction.total_price
                                                        END
                                                    ) as "income: i32",
                                                    construction_shipment.reserved_fund
                                                FROM 
                                                    public.construction_shipment 
                                                    left join public.market_transaction ON market_transaction.construction = construction_shipment.id
                                                group by construction_shipment.id
                                                ORDER BY construction_shipment.id ASC
                                        "#,
                                )
                                .fetch_all(database_pool.get_cache_pool())
                                .await?;
                                Ok(items)
                        },
                        || async move {
                                let count = sqlx::query!(
                                        r#"
                                                SELECT COUNT(*) as "count!"
                                                FROM construction_shipment
                                        "#
                                )
                                .fetch_one(database_pool.get_cache_pool())
                                .await?;
                                Ok(count.count)
                        },
                )
                .await
    }
}

impl DatabaseConnectorAsync for ConstructionShipment {
    type ID = i64;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(
        database_pool: &DbPool,
        item: &ConstructionShipment,
    ) -> crate::Result<Self::ID> {
        let id = sqlx::query!(
            r#"
                                INSERT INTO construction_shipment (
                                    material_id,
                                    construction_site_waypoint,
                                    ship_symbol,
                                    trade_symbol,
                                    units,
                                    purchase_waypoint,
                                    created_at,
                                    updated_at,
                                    status,
                                    reserved_fund
                                )
                                VALUES (
                                    $1, $2, $3, $4::trade_symbol, $5, $6, 
                                    NOW(), NOW(), $7::shipment_status, $8
                                )
                                RETURNING id;
                        "#,
            &item.material_id,
            &item.construction_site_waypoint,
            &item.ship_symbol,
            &item.trade_symbol as &models::TradeSymbol,
            &item.units,
            &item.purchase_waypoint,
            &item.status as &ShipmentStatus,
            &item.reserved_fund as &Option<i64>
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(id.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &ConstructionShipment) -> crate::Result<()> {
        if item.id == 0 {
            let _ = Self::insert_new(database_pool, item).await?;
            return Ok(());
        }

        sqlx::query!(
            r#"
                INSERT INTO construction_shipment (
                  id,
                  material_id,
                  construction_site_waypoint,
                  ship_symbol,
                  trade_symbol,
                  units,
                  purchase_waypoint,
                  created_at,
                  updated_at,
                  status,
                  reserved_fund
                )
                VALUES (
                  $1, $2, $3, $4, $5::trade_symbol, $6, $7, 
                  NOW(), NOW(), $8::shipment_status, $9
                )
                ON CONFLICT (id) DO UPDATE SET
                  material_id = EXCLUDED.material_id,
                  construction_site_waypoint = EXCLUDED.construction_site_waypoint,
                  ship_symbol = EXCLUDED.ship_symbol,
                  trade_symbol = EXCLUDED.trade_symbol,
                  units = EXCLUDED.units,
                  purchase_waypoint = EXCLUDED.purchase_waypoint,
                  updated_at = NOW(),
                  status = EXCLUDED.status,
                  reserved_fund = EXCLUDED.reserved_fund;
            "#,
            &item.id,
            &item.material_id,
            &item.construction_site_waypoint,
            &item.ship_symbol,
            &item.trade_symbol as &models::TradeSymbol,
            &item.units,
            &item.purchase_waypoint,
            &item.status as &ShipmentStatus,
            &item.reserved_fund as &Option<i64>
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, item), err(Debug))]
    async fn update(database_pool: &DbPool, item: &ConstructionShipment) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &DbPool,
        items: &[ConstructionShipment],
    ) -> crate::Result<()> {
        let (
            ids,
            material_ids,
            construction_site_waypoints,
            ship_symbols,
            trade_symbols,
            units_values,
            purchase_waypoints,
            statuses,
            reserved_funds,
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
        ) = itertools::multiunzip(items.iter().map(|cs| {
            (
                cs.id,
                cs.material_id,
                cs.construction_site_waypoint.clone(),
                cs.ship_symbol.clone(),
                cs.trade_symbol,
                cs.units,
                cs.purchase_waypoint.clone(),
                cs.status,
                cs.reserved_fund,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO construction_shipment (
                id,
                material_id,
                construction_site_waypoint,
                ship_symbol,
                trade_symbol,
                units,
                purchase_waypoint,
                created_at,
                updated_at,
                status,
                reserved_fund
            )
            SELECT 
                id,
                mat_id, 
                constr_waypoint, 
                ship, 
                trade, 
                u, 
                purch_waypoint, 
                NOW(), 
                NOW(), 
                stat,
                reserved_fund
            FROM UNNEST(
                $1::bigint[],
                $2::bigint[],
                $3::character varying[],
                $4::character varying[],
                $5::trade_symbol[],
                $6::integer[],
                $7::character varying[],
                $8::shipment_status[],
                $9::bigint[]
            ) AS t(id, mat_id, constr_waypoint, ship, trade, u, purch_waypoint, stat, reserved_fund)
            ON CONFLICT (id) DO UPDATE
            SET material_id = EXCLUDED.material_id,
                construction_site_waypoint = EXCLUDED.construction_site_waypoint,
                ship_symbol = EXCLUDED.ship_symbol,
                trade_symbol = EXCLUDED.trade_symbol,
                units = EXCLUDED.units,
                purchase_waypoint = EXCLUDED.purchase_waypoint,
                updated_at = NOW(),
                status = EXCLUDED.status,
                reserved_fund = EXCLUDED.reserved_fund;
            "#,
            &ids,
            &material_ids,
            &construction_site_waypoints,
            &ship_symbols,
            &trade_symbols as &[models::TradeSymbol],
            &units_values,
            &purchase_waypoints,
            &statuses as &[ShipmentStatus],
            &reserved_funds as &[Option<i64>],
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<ConstructionShipment>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        ORDER BY created_at DESC, id DESC
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
                    ConstructionShipment,
                    r#"
                        SELECT
                          id,
                          material_id,
                          construction_site_waypoint,
                          ship_symbol,
                          trade_symbol as "trade_symbol: models::TradeSymbol",
                          units,
                          purchase_waypoint,
                          created_at,
                          updated_at,
                          status as "status: ShipmentStatus",
                          reserved_fund
                        FROM construction_shipment
                        ORDER BY created_at DESC, id DESC
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
                        FROM construction_shipment
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
            ConstructionShipment,
            r#"
                SELECT
                  id,
                  material_id,
                  construction_site_waypoint,
                  ship_symbol,
                  trade_symbol as "trade_symbol: models::TradeSymbol",
                  units,
                  purchase_waypoint,
                  created_at,
                  updated_at,
                  status as "status: ShipmentStatus",
                  reserved_fund
                FROM construction_shipment
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
                DELETE FROM construction_shipment
                WHERE id = $1
            "#,
            *id
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = id;
    }
}
