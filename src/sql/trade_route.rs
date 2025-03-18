use space_traders_client::models;

use crate::sql;

use super::{DatabaseConnector, DbPool, ShipmentStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub created_at: sqlx::types::chrono::NaiveDateTime,
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
            created_at: sqlx::types::chrono::NaiveDateTime::MIN,
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
        )
    }
}

impl DatabaseConnector<TradeRoute> for TradeRoute {
    async fn insert(database_pool: &DbPool, item: &TradeRoute) -> sqlx::Result<()> {
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
            predicted_sell_price
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
            on conflict (id) do update
            set status = EXCLUDED.status
            "#,
            item.id,
            item.symbol as models::TradeSymbol,
            item.ship_symbol,
            item.purchase_waypoint,
            item.sell_waypoint,
            item.status as sql::ShipmentStatus,
            item.trade_volume,
            item.predicted_purchase_price,
            item.predicted_sell_price
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &[sql::trade_route::TradeRoute]) -> sqlx::Result<()> {
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
        ) = itertools::multiunzip(items.iter().map(|s| {
            (
                s.id,
                s.symbol as models::TradeSymbol,
                s.ship_symbol.clone(),
                s.purchase_waypoint.clone(),
                s.sell_waypoint.clone(),
                s.status as sql::ShipmentStatus,
                s.trade_volume,
                s.predicted_purchase_price,
                s.predicted_sell_price,
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
              predicted_sell_price
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
              $9::integer[]
            )
            on conflict (id) do update
            set status = EXCLUDED.status
            "#,
            &id_s,
            &symbol_s as &[models::TradeSymbol],
            &ship_symbol_s,
            &purchase_waypoint_s,
            &sell_waypoint_s,
            &status_s as &[ShipmentStatus],
            &trade_volume_s,
            &predicted_purchase_price_s,
            &predicted_sell_price_s
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<TradeRoute>> {
        sqlx::query_as!(
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
                  created_at
                FROM trade_route
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl TradeRoute {
    pub async fn insert_new(database_pool: &DbPool, item: &TradeRoute) -> sqlx::Result<i32> {
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
            predicted_sell_price
            ) values (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8
            )
            RETURNING id
            "#,
            item.symbol as models::TradeSymbol,
            item.ship_symbol,
            item.purchase_waypoint,
            item.sell_waypoint,
            item.status as sql::ShipmentStatus,
            item.trade_volume,
            item.predicted_purchase_price,
            item.predicted_sell_price
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        let erg = erg.first().ok_or_else(|| sqlx::Error::RowNotFound)?;

        Ok(erg.id)
    }

    pub async fn get_unfinished(database_pool: &DbPool) -> sqlx::Result<Vec<TradeRoute>> {
        sqlx::query_as!(
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
                  created_at
                 FROM trade_route WHERE status='IN_TRANSIT'
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }

    pub async fn get_summarys(database_pool: &DbPool) -> sqlx::Result<Vec<TradeRouteSummary>> {
        sqlx::query_as!(
            TradeRouteSummary,
            r#"
                SELECT
                id,
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
                ) as "profit: i32"
              FROM
                public.trade_route
              left join public.market_transaction ON market_transaction.trade_route = trade_route.id
              group by
                id
              ORDER BY
                id ASC;
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
