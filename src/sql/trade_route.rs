use space_traders_client::models;

use super::{sql_models::TradeRoute, DatabaseConnector};

impl DatabaseConnector<TradeRoute> for TradeRoute {
    async fn insert(database_pool: &sqlx::PgPool, item: &TradeRoute) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            insert into trade_route (symbol, ship_symbol, purchase_waypoint, sell_waypoint, finished, predicted_purchase_price, predicted_sell_price)
            values ($1, $2, $3, $4, $5, $6, $7)
            on conflict (symbol, ship_symbol, purchase_waypoint, sell_waypoint, predicted_purchase_price, predicted_sell_price) do update
            set finished = EXCLUDED.finished
            "#,
            item.symbol as models::TradeSymbol,
            item.ship_symbol,
            item.purchase_waypoint,
            item.sell_waypoint,
            item.finished,
            item.predicted_purchase_price,
            item.predicted_sell_price
        ).execute(database_pool).await?;

        Ok(())
    }

    async fn insert_bulk(
        database_pool: &sqlx::PgPool,
        items: &Vec<TradeRoute>,
    ) -> sqlx::Result<()> {
        let (
            ((symbol_s, ship_symbol_s), (purchase_waypoint_s, sell_waypoint_s)),
            ((finished_s, predicted_purchase_price_s), (predicted_sell_price_s, ())),
        ): (
            (
                (Vec<models::TradeSymbol>, Vec<String>),
                (Vec<String>, Vec<String>),
            ),
            ((Vec<bool>, Vec<i32>), (Vec<i32>, ())),
        ) = items
            .iter()
            .map(|t| {
                (
                    (
                        (t.symbol, t.ship_symbol.clone()),
                        (t.purchase_waypoint.clone(), t.sell_waypoint.clone()),
                    ),
                    (
                        (t.finished, t.predicted_purchase_price),
                        (t.predicted_sell_price, ()),
                    ),
                )
            })
            .map(
                |f: (
                    ((models::TradeSymbol, String), (String, String)),
                    ((bool, i32), (i32, ())),
                )| f,
            )
            .unzip();
        // .map(|f| f)

        sqlx::query!(
            r#"
            insert into trade_route (
              symbol,
              ship_symbol,
              purchase_waypoint,
              sell_waypoint,
              finished,
              predicted_purchase_price,
              predicted_sell_price
            )
            SELECT * FROM UNNEST(
              $1::trade_symbol[],
              $2::character varying[],
              $3::character varying[],
              $4::character varying[],
              $5::boolean[],
              $6::integer[],
              $7::integer[]
            )
            on conflict (symbol, ship_symbol, purchase_waypoint, sell_waypoint, predicted_purchase_price, predicted_sell_price) do update
            set finished = EXCLUDED.finished
            "#,
            &symbol_s as &[models::TradeSymbol],
            &ship_symbol_s,
            &purchase_waypoint_s,
            &sell_waypoint_s,
            &finished_s,
            &predicted_purchase_price_s,
            &predicted_sell_price_s
        )
        .execute(database_pool)
        .await?;

        Ok(())
    }

    async fn get_all(database_pool: &sqlx::PgPool) -> sqlx::Result<Vec<TradeRoute>> {
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
                  predicted_purchase_price,
                  predicted_sell_price,
                  created_at
                FROM trade_route
            "#
        )
        .fetch_all(database_pool)
        .await
    }
}
