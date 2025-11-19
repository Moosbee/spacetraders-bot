use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnector, DbPool, ShipmentStatus};

#[derive(Debug, Clone, PartialEq, Eq, async_graphql::SimpleObject)]
#[graphql(name = "DBTradeRoute")]
pub struct TradeRoute {
    pub id: i32,
    pub symbol: models::TradeSymbol,
    pub ship_symbol: String,
    pub purchase_waypoint: String,
    pub sell_waypoint: String,
    pub status: ShipmentStatus,
    pub trade_volume: i32,
    pub predicted_purchase_price: i32,
    pub predicted_sell_price: i32,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub reserved_fund: Option<i64>,
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
            predicted_purchase_price: 0,
            predicted_sell_price: 0,
            created_at: sqlx::types::chrono::DateTime::<chrono::Utc>::MIN_UTC,
            reserved_fund: None,
        }
    }
}

impl std::fmt::Display for TradeRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}: {} -> {} {}",
            self.ship_symbol,
            self.symbol,
            self.purchase_waypoint,
            self.sell_waypoint,
            self.trade_volume * self.predicted_sell_price
                - self.predicted_purchase_price * self.trade_volume
        )
    }
}

impl DatabaseConnector<TradeRoute> for TradeRoute {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &DbPool, item: &TradeRoute) -> crate::Result<()> {
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
            predicted_purchase_price,
            predicted_sell_price,
            reserved_fund
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
            $10
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
            item.predicted_purchase_price,
            item.predicted_sell_price,
            item.reserved_fund
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &DbPool,
        items: &[crate::trade_route::TradeRoute],
    ) -> crate::Result<()> {
        let (
            id_s,
            symbol_s,
            ship_symbol_s,
            purchase_waypoint_s,
            sell_waypoint_s,
            status_s,
            trade_volume_s,
            predicted_purchase_price_s,
            predicted_sell_price_s,
            reserved_fund_s,
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
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|s| {
            (
                s.id,
                s.symbol as models::TradeSymbol,
                s.ship_symbol.clone(),
                s.purchase_waypoint.clone(),
                s.sell_waypoint.clone(),
                s.status as crate::ShipmentStatus,
                s.trade_volume,
                s.predicted_purchase_price,
                s.predicted_sell_price,
                s.reserved_fund,
            )
        }));

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
              predicted_purchase_price,
              predicted_sell_price,
              reserved_fund
            )
            SELECT * FROM UNNEST(
              $1::integer[],
              $2::trade_symbol[],
              $3::character varying[],
              $4::character varying[],
              $5::character varying[],
              $6::shipment_status[],
              $7::integer[],
              $8::integer[],
              $9::integer[],
              $10::bigint[]
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
            &predicted_purchase_price_s,
            &predicted_sell_price_s,
            &reserved_fund_s as &[Option<i64>],
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<TradeRoute>> {
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
                  predicted_purchase_price,
                  predicted_sell_price,
                  created_at,
                  reserved_fund
                FROM trade_route
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl TradeRoute {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn insert_new(database_pool: &DbPool, item: &TradeRoute) -> crate::Result<i32> {
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
            predicted_purchase_price,
            predicted_sell_price,
            reserved_fund
            ) values (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9
            )
            RETURNING id
            "#,
            item.symbol as models::TradeSymbol,
            item.ship_symbol,
            item.purchase_waypoint,
            item.sell_waypoint,
            item.status as crate::ShipmentStatus,
            item.trade_volume,
            item.predicted_purchase_price,
            item.predicted_sell_price,
            item.reserved_fund
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        let erg = erg.first().ok_or_else(|| sqlx::Error::RowNotFound)?;

        Ok(erg.id)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_id(database_pool: &DbPool, id: i32) -> crate::Result<Option<TradeRoute>> {
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
                  predicted_purchase_price,
                  predicted_sell_price,
                  created_at,
                  reserved_fund
                 FROM trade_route WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_unfinished(database_pool: &DbPool) -> crate::Result<Vec<TradeRoute>> {
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
                  predicted_purchase_price,
                  predicted_sell_price,
                  created_at,
                  reserved_fund
                 FROM trade_route WHERE status='IN_TRANSIT'
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }

    pub async fn get_by_ship(
        database_pool: &DbPool,
        ship_symbol: &str,
    ) -> crate::Result<Vec<TradeRoute>> {
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
                  predicted_purchase_price,
                  predicted_sell_price,
                  created_at,
                  reserved_fund
                 FROM trade_route WHERE ship_symbol = $1
            "#,
            ship_symbol
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_summarys(database_pool: &DbPool) -> crate::Result<Vec<TradeRouteSummary>> {
        let erg= sqlx::query_as!(
            TradeRouteSummary,
            r#"
                SELECT
                trade_route.id,
                symbol as "symbol: models::TradeSymbol",
                trade_route.ship_symbol,
                purchase_waypoint,
                sell_waypoint,
                status as "status: ShipmentStatus",
                trade_volume,
                predicted_purchase_price,
                predicted_sell_price,
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
                sum(
                  CASE
                    WHEN market_transaction.type = 'PURCHASE' THEN (market_transaction.total_price * -1)
                    ELSE market_transaction.total_price
                  END
                ) as "profit: i32",
                reserved_fund
              FROM
                public.trade_route
              left join public.market_transaction ON market_transaction.trade_route = trade_route.id
              group by
                trade_route.id
              ORDER BY
                trade_route.id ASC;
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
