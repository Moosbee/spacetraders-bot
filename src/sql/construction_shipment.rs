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
    pub created_at: sqlx::types::chrono::NaiveDateTime,
    pub updated_at: sqlx::types::chrono::NaiveDateTime,
    pub status: ShipmentStatus,
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
}

impl DatabaseConnector<ConstructionShipment> for ConstructionShipment {
    async fn insert(database_pool: &DbPool, item: &ConstructionShipment) -> sqlx::Result<()> {
        sqlx::query!(
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
        items: &Vec<ConstructionShipment>,
    ) -> sqlx::Result<()> {
        let (
            material_ids,
            construction_site_waypoints,
            ship_symbols,
            trade_symbols,
            units_values,
            purchase_waypoints,
            statuses,
        ): (
            Vec<i64>,
            Vec<String>,
            Vec<String>,
            Vec<models::TradeSymbol>,
            Vec<i32>,
            Vec<String>,
            Vec<ShipmentStatus>,
        ) = itertools::multiunzip(items.iter().map(|cs| {
            (
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
                $2::character varying[],
                $3::character varying[],
                $4::trade_symbol[],
                $5::integer[],
                $6::character varying[],
                $7::shipment_status[]
            ) AS t(mat_id, constr_waypoint, ship, trade, u, purch_waypoint, stat)
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
