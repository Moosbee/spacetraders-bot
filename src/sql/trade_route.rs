use space_traders_client::models;

use super::{
    sql_models::{TradeRoute, TradeRouteSummary},
    DatabaseConnector, DbPool,
};

impl DatabaseConnector<TradeRoute> for TradeRoute {
    async fn insert(database_pool: &DbPool, item: &TradeRoute) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            insert into trade_route (id, symbol, ship_symbol, purchase_waypoint, sell_waypoint, finished, trade_volume, predicted_purchase_price, predicted_sell_price)
            values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            on conflict (id) do update
            set finished = EXCLUDED.finished
            "#,
            item.id,
            item.symbol as models::TradeSymbol,
            item.ship_symbol,
            item.purchase_waypoint,
            item.sell_waypoint,
            item.finished,
            item.trade_volume,
            item.predicted_purchase_price,
            item.predicted_sell_price
        ).execute(&database_pool.database_pool).await?;

        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &Vec<TradeRoute>) -> sqlx::Result<()> {
        let (
            ((symbol_s, ship_symbol_s), (purchase_waypoint_s, sell_waypoint_s)),
            (
                (finished_and_trade_volume_s, predicted_purchase_price_s),
                (predicted_sell_price_s, id_s),
            ),
        ): (
            (
                (Vec<models::TradeSymbol>, Vec<String>),
                (Vec<String>, Vec<String>),
            ),
            ((Vec<(bool, i32)>, Vec<i32>), (Vec<i32>, Vec<i32>)),
        ) = items
            .iter()
            .map(|t| {
                (
                    (
                        (t.symbol, t.ship_symbol.clone()),
                        (t.purchase_waypoint.clone(), t.sell_waypoint.clone()),
                    ),
                    (
                        ((t.finished, t.trade_volume), t.predicted_purchase_price),
                        (t.predicted_sell_price, t.id),
                    ),
                )
            })
            .map(
                |f: (
                    ((models::TradeSymbol, String), (String, String)),
                    (((bool, i32), i32), (i32, i32)),
                )| f,
            )
            .unzip();
        // .map(|f| f)

        let (finished_s, trade_volume_s): (Vec<bool>, Vec<i32>) =
            finished_and_trade_volume_s.into_iter().unzip();

        sqlx::query!(
            r#"
            insert into trade_route (
              id,
              symbol,
              ship_symbol,
              purchase_waypoint,
              sell_waypoint,
              finished,
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
              $6::boolean[],
              $7::integer[],
              $8::integer[],
              $9::integer[]
            )
            on conflict (id) do update
            set finished = EXCLUDED.finished
            "#,
            &id_s,
            &symbol_s as &[models::TradeSymbol],
            &ship_symbol_s,
            &purchase_waypoint_s,
            &sell_waypoint_s,
            &finished_s,
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
                  finished,
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
        let erg= sqlx::query_as!(
            Erg,
            r#"
            insert into trade_route (symbol, ship_symbol, purchase_waypoint, sell_waypoint, finished,trade_volume, predicted_purchase_price, predicted_sell_price)
            values ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            item.symbol as models::TradeSymbol,
            item.ship_symbol,
            item.purchase_waypoint,
            item.sell_waypoint,
            item.finished,
            item.trade_volume,
            item.predicted_purchase_price,
            item.predicted_sell_price
        ).fetch_all(&database_pool.database_pool).await?;

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
                  finished,
                  trade_volume,
                  predicted_purchase_price,
                  predicted_sell_price,
                  created_at
                 FROM trade_route WHERE finished=false
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
  finished,
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
