use space_traders_client::models;

use super::{DatabaseConnector, DbPool, ShipmentStatus};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConstructionShipment {
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
}

impl ConstructionShipment {
    pub(crate) async fn insert_new(
        database_pool: &DbPool,
        next_shipment: &ConstructionShipment,
    ) -> sqlx::Result<i64> {
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
                  status
                )
                VALUES (
                  $1, $2, $3, $4::trade_symbol, $5, $6, 
                  NOW(), NOW(), $7::shipment_status
                )
                RETURNING id;
            "#,
            &next_shipment.material_id,
            &next_shipment.construction_site_waypoint,
            &next_shipment.ship_symbol,
            &next_shipment.trade_symbol as &models::TradeSymbol,
            &next_shipment.units,
            &next_shipment.purchase_waypoint,
            &next_shipment.status as &ShipmentStatus
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(id.id)
    }

    pub(crate) async fn get_by_id(
        database_pool: &DbPool,
        id: i64,
    ) -> sqlx::Result<Option<ConstructionShipment>> {
        sqlx::query_as!(
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
                  status as "status: ShipmentStatus"
                FROM construction_shipment
                WHERE id = $1
                LIMIT 1
            "#,
            id
        )
        .fetch_optional(&database_pool.database_pool)
        .await
    }

    pub async fn get_all_in_transit(
        database_pool: &DbPool,
    ) -> sqlx::Result<Vec<ConstructionShipment>> {
        sqlx::query_as!(
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
                  status as "status: ShipmentStatus"
                FROM construction_shipment
                WHERE status = 'IN_TRANSIT'
            "#,
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }

    pub(crate) async fn get_summary(
        database_pool: &DbPool,
    ) -> sqlx::Result<Vec<ConstructionShipmentSummary>> {
        sqlx::query_as!(
            ConstructionShipmentSummary,
            r#"
                SELECT
                  id,
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
                  ) as "income: i32"
                FROM 
                  public.construction_shipment 
                  left join public.market_transaction ON market_transaction.construction = construction_shipment.id
                group by
                  id
                ORDER BY
                  id ASC;
            "#,
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl DatabaseConnector<ConstructionShipment> for ConstructionShipment {
    async fn insert(database_pool: &DbPool, item: &ConstructionShipment) -> sqlx::Result<()> {
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
                  status
                )
                VALUES (
                  $1, $2, $3, $4, $5::trade_symbol, $6, $7, 
                  NOW(), NOW(), $8::shipment_status
                )
                ON CONFLICT (id) DO UPDATE SET
                  material_id = EXCLUDED.material_id,
                  construction_site_waypoint = EXCLUDED.construction_site_waypoint,
                  ship_symbol = EXCLUDED.ship_symbol,
                  trade_symbol = EXCLUDED.trade_symbol,
                  units = EXCLUDED.units,
                  purchase_waypoint = EXCLUDED.purchase_waypoint,
                  updated_at = NOW(),
                  status = EXCLUDED.status;
            "#,
            &item.id,
            &item.material_id,
            &item.construction_site_waypoint,
            &item.ship_symbol,
            &item.trade_symbol as &models::TradeSymbol,
            &item.units,
            &item.purchase_waypoint,
            &item.status as &ShipmentStatus
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(
        database_pool: &DbPool,
        items: &[ConstructionShipment],
    ) -> sqlx::Result<()> {
        let (
            ids,
            material_ids,
            construction_site_waypoints,
            ship_symbols,
            trade_symbols,
            units_values,
            purchase_waypoints,
            statuses,
        ): (
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
                status
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
                stat 
            FROM UNNEST(
                $1::bigint[],
                $2::bigint[],
                $3::character varying[],
                $4::character varying[],
                $5::trade_symbol[],
                $6::integer[],
                $7::character varying[],
                $8::shipment_status[]
            ) AS t(id, mat_id, constr_waypoint, ship, trade, u, purch_waypoint, stat)
            ON CONFLICT (id) DO UPDATE
            SET material_id = EXCLUDED.material_id,
                construction_site_waypoint = EXCLUDED.construction_site_waypoint,
                ship_symbol = EXCLUDED.ship_symbol,
                trade_symbol = EXCLUDED.trade_symbol,
                units = EXCLUDED.units,
                purchase_waypoint = EXCLUDED.purchase_waypoint,
                updated_at = NOW(),
                status = EXCLUDED.status;
            "#,
            &ids,
            &material_ids,
            &construction_site_waypoints,
            &ship_symbols,
            &trade_symbols as &[models::TradeSymbol],
            &units_values,
            &purchase_waypoints,
            &statuses as &[ShipmentStatus]
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<ConstructionShipment>> {
        sqlx::query_as!(
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
                  status as "status: ShipmentStatus"
                FROM construction_shipment
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
