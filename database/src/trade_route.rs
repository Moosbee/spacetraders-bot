use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::Loader;
use space_traders_client::models;
use tracing::instrument;

use super::{
    DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, ShipmentStatus,
    run_paginated_query,
};

pub struct TradeRouteLoader(DbPool);

impl TradeRouteLoader {
    pub fn new(database_pool: DbPool) -> Self {
        Self(database_pool)
    }

    async fn get_by_ids(database_pool: &DbPool, ids: &[i32]) -> crate::Result<Vec<TradeRoute>> {
        let erg = sqlx::query_as!(
            TradeRoute,
            r#"
                SELECT
                  id,
                  symbol as "symbol: models::TradeSymbol",
                  ship_symbol,
                  purchase_waypoint,
                  sell_waypoint,
                  status as "status: ShipmentStatus",
                  trade_volume,
                  purchase_trade_good_id,
                  sell_trade_good_id,
                  estimated_fuel,
                  trade_mode as "trade_mode: crate::TradeMode",
                  reserved_fund,
                  fleet_id,
                  assignment_id,
                  created_at
                 FROM trade_route WHERE id = ANY($1)
            "#,
            &ids
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl Loader<i32> for TradeRouteLoader {
    type Value = TradeRoute;
    type Error = Arc<crate::Error>;

    #[instrument(level = "trace", skip(self, keys))]
    async fn load(
        &self,
        keys: &[i32],
    ) -> std::result::Result<HashMap<i32, Self::Value>, Self::Error> {
        let ids: Vec<i32> = keys.to_vec();
        let mut map = HashMap::new();
        for trade_route in Self::get_by_ids(&self.0, &ids).await? {
            map.insert(trade_route.id, trade_route);
        }
        Ok(map)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, async_graphql::SimpleObject)]
#[graphql(name = "DBTradeRoute")]
pub struct TradeRoute {
    pub id: i32,
    pub symbol: models::TradeSymbol,
    pub ship_symbol: String,
    #[graphql(name = "PurchaseWaypointSymbol")]
    pub purchase_waypoint: String,
    #[graphql(name = "SellWaypointSymbol")]
    pub sell_waypoint: String,
    pub status: ShipmentStatus,
    pub trade_volume: i32,
    pub purchase_trade_good_id: Option<i64>,
    pub sell_trade_good_id: Option<i64>,
    pub estimated_fuel: Option<i32>,
    pub trade_mode: crate::TradeMode,
    pub reserved_fund: Option<i64>,
    pub fleet_id: Option<i32>,
    pub assignment_id: Option<i64>,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TradeRouteSummary {
    pub id: i32,
    pub symbol: models::TradeSymbol,
    pub ship_symbol: String,
    pub purchase_waypoint: String,
    pub sell_waypoint: String,
    pub status: ShipmentStatus,
    pub trade_volume: i32,
    pub predicted_purchase_price: i32,
    pub predicted_sell_price: i32,
    pub sum: Option<i32>,
    pub expenses: Option<i32>,
    pub income: Option<i32>,
    pub profit: Option<i32>,
    pub reserved_fund: Option<i64>,
}

impl TradeRoute {
    pub fn complete(self) -> Self {
        TradeRoute {
            status: ShipmentStatus::Delivered,
            ..self
        }
    }
}

impl Default for TradeRoute {
    fn default() -> TradeRoute {
        TradeRoute {
            id: 0,
            symbol: models::TradeSymbol::PreciousStones,
            ship_symbol: String::new(),
            purchase_waypoint: String::new(),
            sell_waypoint: String::new(),
            status: ShipmentStatus::InTransit,
            trade_volume: 0,
            purchase_trade_good_id: None,
            sell_trade_good_id: None,
            estimated_fuel: None,
            trade_mode: crate::TradeMode::default(),
            reserved_fund: None,
            fleet_id: None,
            assignment_id: None,
            created_at: sqlx::types::chrono::DateTime::<chrono::Utc>::MIN_UTC,
        }
    }
}

impl std::fmt::Display for TradeRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}: {} -> {}",
            self.ship_symbol, self.symbol, self.purchase_waypoint, self.sell_waypoint,
        )
    }
}

impl DatabaseConnectorAsync for TradeRoute {
    type ID = i32;

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &DbPool, item: &TradeRoute) -> crate::Result<Self::ID> {
        struct Erg {
            id: i32,
        }

        let erg = sqlx::query_as!(
            Erg,
            r#"
            insert into trade_route (
            symbol,
            ship_symbol,
            purchase_waypoint,
            sell_waypoint,
            status,
            trade_volume,
            purchase_trade_good_id,
            sell_trade_good_id,
            estimated_fuel,
            trade_mode,
            reserved_fund,
            fleet_id,
            assignment_id
            ) values (
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
            $13
            )
            RETURNING id
            "#,
            item.symbol as models::TradeSymbol,
            item.ship_symbol,
            item.purchase_waypoint,
            item.sell_waypoint,
            item.status as crate::ShipmentStatus,
            item.trade_volume,
            item.purchase_trade_good_id,
            item.sell_trade_good_id,
            item.estimated_fuel,
            item.trade_mode as crate::TradeMode,
            item.reserved_fund,
            item.fleet_id,
            item.assignment_id
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        let erg = erg.first().ok_or_else(|| sqlx::Error::RowNotFound)?;

        Ok(erg.id)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &TradeRoute) -> crate::Result<()> {
        if item.id == 0 {
            let _ = Self::insert_new(database_pool, item).await?;
            return Ok(());
        }

        sqlx::query!(
            r#"
            insert into trade_route (
            id,
            symbol,
            ship_symbol,
            purchase_waypoint,
            sell_waypoint,
            status,
            trade_volume,
            purchase_trade_good_id,
            sell_trade_good_id,
            estimated_fuel,
            trade_mode,
            reserved_fund,
            fleet_id,
            assignment_id
            ) values (
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
            $14
            )
            on conflict (id) do update
            set status = EXCLUDED.status,
            reserved_fund = EXCLUDED.reserved_fund
            "#,
            item.id,
            item.symbol as models::TradeSymbol,
            item.ship_symbol,
            item.purchase_waypoint,
            item.sell_waypoint,
            item.status as crate::ShipmentStatus,
            item.trade_volume,
            item.purchase_trade_good_id,
            item.sell_trade_good_id,
            item.estimated_fuel,
            item.trade_mode as crate::TradeMode,
            item.reserved_fund,
            item.fleet_id,
            item.assignment_id
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &TradeRoute) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &DbPool,
        items: &[crate::trade_route::TradeRoute],
    ) -> crate::Result<()> {
        let mut id_s: Vec<i32> = Vec::with_capacity(items.len());
        let mut symbol_s: Vec<models::TradeSymbol> = Vec::with_capacity(items.len());
        let mut ship_symbol_s: Vec<String> = Vec::with_capacity(items.len());
        let mut purchase_waypoint_s: Vec<String> = Vec::with_capacity(items.len());
        let mut sell_waypoint_s: Vec<String> = Vec::with_capacity(items.len());
        let mut status_s: Vec<ShipmentStatus> = Vec::with_capacity(items.len());
        let mut trade_volume_s: Vec<i32> = Vec::with_capacity(items.len());
        let mut purchase_trade_good_id_s: Vec<Option<i64>> = Vec::with_capacity(items.len());
        let mut sell_trade_good_id_s: Vec<Option<i64>> = Vec::with_capacity(items.len());
        let mut estimated_fuel_s: Vec<Option<i32>> = Vec::with_capacity(items.len());
        let mut trade_mode_s: Vec<crate::TradeMode> = Vec::with_capacity(items.len());
        let mut reserved_fund_s: Vec<Option<i64>> = Vec::with_capacity(items.len());
        let mut fleet_id_s: Vec<Option<i32>> = Vec::with_capacity(items.len());
        let mut assignment_id_s: Vec<Option<i64>> = Vec::with_capacity(items.len());

        for s in items {
            id_s.push(s.id);
            symbol_s.push(s.symbol as models::TradeSymbol);
            ship_symbol_s.push(s.ship_symbol.clone());
            purchase_waypoint_s.push(s.purchase_waypoint.clone());
            sell_waypoint_s.push(s.sell_waypoint.clone());
            status_s.push(s.status as crate::ShipmentStatus);
            trade_volume_s.push(s.trade_volume);
            purchase_trade_good_id_s.push(s.purchase_trade_good_id);
            sell_trade_good_id_s.push(s.sell_trade_good_id);
            estimated_fuel_s.push(s.estimated_fuel);
            trade_mode_s.push(s.trade_mode as crate::TradeMode);
            reserved_fund_s.push(s.reserved_fund);
            fleet_id_s.push(s.fleet_id);
            assignment_id_s.push(s.assignment_id);
        }

        sqlx::query!(
            r#"
            insert into trade_route (
              id,
              symbol,
              ship_symbol,
              purchase_waypoint,
              sell_waypoint,
              status,
              trade_volume,
              purchase_trade_good_id,
              sell_trade_good_id,
              estimated_fuel,
              trade_mode,
              reserved_fund,
              fleet_id,
              assignment_id
            )
            SELECT * FROM UNNEST(
              $1::integer[],
              $2::trade_symbol[],
              $3::character varying[],
              $4::character varying[],
              $5::character varying[],
              $6::shipment_status[],
              $7::integer[],
              $8::bigint[],
              $9::bigint[],
              $10::bigint[],
              $11::trade_mode[],
              $12::bigint[],
              $13::integer[],
              $14::bigint[]
            )
            on conflict (id) do update
            set status = EXCLUDED.status,
            reserved_fund = EXCLUDED.reserved_fund
            "#,
            &id_s,
            &symbol_s as &[models::TradeSymbol],
            &ship_symbol_s,
            &purchase_waypoint_s,
            &sell_waypoint_s,
            &status_s as &[ShipmentStatus],
            &trade_volume_s,
            &purchase_trade_good_id_s as &[Option<i64>],
            &sell_trade_good_id_s as &[Option<i64>],
            &estimated_fuel_s as &[Option<i32>],
            &trade_mode_s as &[crate::TradeMode],
            &reserved_fund_s as &[Option<i64>],
            &fleet_id_s as &[Option<i32>],
            &assignment_id_s as &[Option<i64>],
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                        FROM trade_route
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
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                        FROM trade_route
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
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
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
            TradeRoute,
            r#"
                SELECT 
                  id,
                  symbol as "symbol: models::TradeSymbol",
                  ship_symbol,
                  purchase_waypoint,
                  sell_waypoint,
                  status as "status: ShipmentStatus",
                  trade_volume,
                  purchase_trade_good_id,
                  sell_trade_good_id,
                  estimated_fuel,
                  trade_mode as "trade_mode: crate::TradeMode",
                  reserved_fund,
                  fleet_id,
                  assignment_id,
                  created_at
                 FROM trade_route WHERE id = $1
            "#,
            *id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM trade_route
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

impl TradeRoute {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_reservation_id(
        database_pool: &DbPool,
        id: i64,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE reserved_fund = $1
                         ORDER BY created_at DESC, id DESC
                         LIMIT $2 OFFSET $3
                    "#,
                    id,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE reserved_fund = $1
                         ORDER BY created_at DESC, id DESC
                    "#,
                    id
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
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
    pub async fn get_unfinished(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE status = 'IN_TRANSIT'
                         ORDER BY created_at DESC, id DESC
                         LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE status = 'IN_TRANSIT'
                         ORDER BY created_at DESC, id DESC
                    "#
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
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
    pub async fn get_by_ship(
        database_pool: &DbPool,
        ship_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE ship_symbol = $1
                         ORDER BY created_at DESC, id DESC
                         LIMIT $2 OFFSET $3
                    "#,
                    ship_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE ship_symbol = $1
                         ORDER BY created_at DESC, id DESC
                    "#,
                    ship_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
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

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_purchase_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE purchase_waypoint = $1
                         ORDER BY created_at DESC, id DESC
                         LIMIT $2 OFFSET $3
                    "#,
                    waypoint_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE purchase_waypoint = $1
                         ORDER BY created_at DESC, id DESC
                    "#,
                    waypoint_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
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
    pub async fn get_by_sell_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE sell_waypoint = $1
                         ORDER BY created_at DESC, id DESC
                         LIMIT $2 OFFSET $3
                    "#,
                    waypoint_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE sell_waypoint = $1
                         ORDER BY created_at DESC, id DESC
                    "#,
                    waypoint_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
                        WHERE sell_waypoint = $1
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
    pub async fn get_by_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE sell_waypoint = $1 OR purchase_waypoint = $1
                         ORDER BY created_at DESC, id DESC
                         LIMIT $2 OFFSET $3
                    "#,
                    waypoint_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE sell_waypoint = $1 OR purchase_waypoint = $1
                         ORDER BY created_at DESC, id DESC
                    "#,
                    waypoint_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
                        WHERE sell_waypoint = $1 OR purchase_waypoint = $1
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
    pub async fn get_by_purchase_system(
        database_pool: &DbPool,
        system_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE purchase_waypoint LIKE ($1 || '-%')
                         ORDER BY created_at DESC, id DESC
                         LIMIT $2 OFFSET $3
                    "#,
                    system_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE purchase_waypoint LIKE ($1 || '-%')
                         ORDER BY created_at DESC, id DESC
                    "#,
                    system_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
                        WHERE purchase_waypoint LIKE ($1 || '-%')
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
    pub async fn get_by_sell_system(
        database_pool: &DbPool,
        system_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE sell_waypoint LIKE ($1 || '-%')
                         ORDER BY created_at DESC, id DESC
                         LIMIT $2 OFFSET $3
                    "#,
                    system_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE sell_waypoint LIKE ($1 || '-%')
                         ORDER BY created_at DESC, id DESC
                    "#,
                    system_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
                        WHERE sell_waypoint LIKE ($1 || '-%')
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
    pub async fn get_by_system(
        database_pool: &DbPool,
        system_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<TradeRoute>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE sell_waypoint LIKE ($1 || '-%') OR purchase_waypoint LIKE ($1 || '-%')
                         ORDER BY created_at DESC, id DESC
                         LIMIT $2 OFFSET $3
                    "#,
                    system_symbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    TradeRoute,
                    r#"
                        SELECT 
                          id,
                          symbol as "symbol: models::TradeSymbol",
                          ship_symbol,
                          purchase_waypoint,
                          sell_waypoint,
                          status as "status: ShipmentStatus",
                          trade_volume,
                          purchase_trade_good_id,
                          sell_trade_good_id,
                          estimated_fuel,
                          trade_mode as "trade_mode: crate::TradeMode",
                          reserved_fund,
                          fleet_id,
                          assignment_id,
                          created_at
                         FROM trade_route
                         WHERE sell_waypoint LIKE ($1 || '-%') OR purchase_waypoint LIKE ($1 || '-%')
                         ORDER BY created_at DESC, id DESC
                    "#,
                    system_symbol
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(1) as "count!"
                        FROM trade_route
                        WHERE sell_waypoint LIKE ($1 || '-%') OR purchase_waypoint LIKE ($1 || '-%')
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
}
