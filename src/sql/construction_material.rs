use chrono::NaiveDateTime;
use space_traders_client::models;

use super::{DatabaseConnector, DbPool};

pub struct ConstructionMaterial {
    pub id: i64,
    pub waypoint_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub required: i32,
    pub fulfilled: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl DatabaseConnector<ConstructionMaterial> for ConstructionMaterial {
    async fn insert(database_pool: &DbPool, item: &ConstructionMaterial) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO construction_material (
                  waypoint_symbol,
                  trade_symbol,
                  required,
                  fulfilled,
                  updated_at
                )
                VALUES ($1, $2::trade_symbol, $3, $4, NOW())
                ON CONFLICT (waypoint_symbol, trade_symbol) DO UPDATE SET
                  required = EXCLUDED.required,
                  fulfilled = EXCLUDED.fulfilled,
                  updated_at = NOW();
            "#,
            &item.waypoint_symbol,
            &item.trade_symbol as &models::TradeSymbol,
            &item.required,
            &item.fulfilled
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(
        database_pool: &DbPool,
        items: &Vec<ConstructionMaterial>,
    ) -> sqlx::Result<()> {
        let (waypoint_symbols, trade_symbols, requireds, fulfilleds): (
            Vec<String>,
            Vec<models::TradeSymbol>,
            Vec<i32>,
            Vec<i32>,
        ) = itertools::multiunzip(items.iter().map(|cm| {
            (
                cm.waypoint_symbol.clone(),
                cm.trade_symbol,
                cm.required,
                cm.fulfilled,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO construction_material (
                waypoint_symbol,
                trade_symbol,
                required,
                fulfilled,
                updated_at
            )
            SELECT waypoint, trade, req, ful, NOW() FROM UNNEST(
                $1::character varying[],
                $2::trade_symbol[],
                $3::integer[],
                $4::integer[]
            ) AS t(waypoint, trade, req, ful)
            ON CONFLICT (waypoint_symbol, trade_symbol) DO UPDATE
            SET required = EXCLUDED.required,
                fulfilled = EXCLUDED.fulfilled,
                updated_at = NOW();
            "#,
            &waypoint_symbols,
            &trade_symbols as &[models::TradeSymbol],
            &requireds,
            &fulfilleds
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<ConstructionMaterial>> {
        sqlx::query_as!(
            ConstructionMaterial,
            r#"
                SELECT
                  id,
                  waypoint_symbol,
                  trade_symbol as "trade_symbol: models::TradeSymbol",
                  required,
                  fulfilled,
                  created_at,
                  updated_at
                FROM construction_material
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
