use chrono::{DateTime, Utc};
use space_traders_client::models::{self};

use super::{DatabaseConnector, DbPool};

pub struct Extraction {
    #[allow(dead_code)]
    pub id: i64,
    pub ship_symbol: String,
    pub waypoint_symbol: String,
    pub ship_info_before: i64,
    pub ship_info_after: i64,
    pub siphon: bool,
    pub yield_symbol: models::TradeSymbol,
    pub yield_units: i32,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl DatabaseConnector<Extraction> for Extraction {
    async fn insert(database_pool: &DbPool, item: &Extraction) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO extraction (
                  ship_symbol,
                  waypoint_symbol,
                  ship_info_before,
                  ship_info_after,
                  siphon,
                  yield_symbol,
                  yield_units
                )
                VALUES (
                  $1,
                  $2,
                  $3,
                  $4,
                  $5,
                  $6,
                  $7
                )
                ON CONFLICT (id) DO UPDATE SET
                  ship_symbol = EXCLUDED.ship_symbol,
                  waypoint_symbol = EXCLUDED.waypoint_symbol,
                  ship_info_before = EXCLUDED.ship_info_before,
                  ship_info_after = EXCLUDED.ship_info_after,
                  siphon = EXCLUDED.siphon,
                  yield_symbol = EXCLUDED.yield_symbol,
                  yield_units = EXCLUDED.yield_units;
            "#,
            &item.ship_symbol,
            &item.waypoint_symbol,
            &item.ship_info_before,
            &item.ship_info_after,
            &item.siphon,
            &item.yield_symbol as &models::TradeSymbol,
            &item.yield_units
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &[Extraction]) -> sqlx::Result<()> {
        let (
            ship_symbols,
            waypoint_symbols,
            ship_info_befores,
            ship_info_afters,
            siphons,
            yield_symbols,
            yield_units,
        ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
            itertools::multiunzip(items.iter().map(|e| {
                (
                    e.ship_symbol.clone(),
                    e.waypoint_symbol.clone(),
                    e.ship_info_before,
                    e.ship_info_after,
                    e.siphon,
                    e.yield_symbol,
                    e.yield_units,
                )
            }));

        sqlx::query!(
            r#"
            INSERT INTO extraction (
                ship_symbol,
                waypoint_symbol,
                ship_info_before,
                ship_info_after,
                siphon,
                yield_symbol,
                yield_units
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::bigint[],
                $4::bigint[],
                $5::boolean[],
                $6::trade_symbol[],
                $7::integer[]
            )
            ON CONFLICT (id) DO UPDATE
            SET ship_symbol = EXCLUDED.ship_symbol,
                waypoint_symbol = EXCLUDED.waypoint_symbol,
                ship_info_before = EXCLUDED.ship_info_before,
                ship_info_after = EXCLUDED.ship_info_after,
                siphon = EXCLUDED.siphon,
                yield_symbol = EXCLUDED.yield_symbol,
                yield_units = EXCLUDED.yield_units;
            "#,
            &ship_symbols,
            &waypoint_symbols,
            &ship_info_befores,
            &ship_info_afters,
            &siphons,
            &yield_symbols as &[models::TradeSymbol],
            &yield_units
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<Extraction>> {
        sqlx::query_as!(
            Extraction,
            r#"
                SELECT
                  id,
                  ship_symbol,
                  waypoint_symbol,
                  ship_info_before,
                  ship_info_after,
                  siphon,
                  yield_symbol as "yield_symbol: models::TradeSymbol",
                  yield_units,
                  created_at
                FROM extraction
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
