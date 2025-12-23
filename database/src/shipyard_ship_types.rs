use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::DatabaseConnector;

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBShipyardShipTypes")]
pub struct ShipyardShipTypes {
    #[allow(dead_code)]
    pub id: i64,
    pub shipyard_id: i64,
    pub ship_type: models::ShipType,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl ShipyardShipTypes {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<ShipyardShipTypes>> {
        let erg= sqlx::query_as!(
            ShipyardShipTypes,
            r#"
            SELECT
                id,
                shipyard_id,
                ship_type as "ship_type: models::ShipType",
                created_at
            FROM shipyard_ship_types
            WHERE shipyard_id = (SELECT id FROM shipyard WHERE waypoint_symbol = $1 ORDER BY created_at DESC LIMIT 1)
            "#,
            waypoint_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    pub async fn get_last_by_system(
        database_pool: &super::DbPool,
        system_symbol: &str,
    ) -> crate::Result<Vec<ShipyardShipTypes>> {
        let erg = sqlx::query_as!(
            ShipyardShipTypes,
            r#"
            SELECT
                id,
                shipyard_id,
                ship_type as "ship_type: models::ShipType",
                shipyard_ship_types.created_at
            FROM shipyard_ship_types
            WHERE shipyard_id = ANY(
                SELECT DISTINCT ON (shipyard.waypoint_symbol) shipyard.id FROM shipyard JOIN waypoint ON shipyard.waypoint_symbol = waypoint.symbol
                WHERE waypoint.system_symbol = $1
                ORDER BY shipyard.waypoint_symbol, shipyard.created_at DESC
            )
            "#,
            system_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnector<ShipyardShipTypes> for ShipyardShipTypes {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert(database_pool: &super::DbPool, item: &ShipyardShipTypes) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO shipyard_ship_types (
                    shipyard_id,
                    ship_type
                )
                VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE
                SET shipyard_id = EXCLUDED.shipyard_id,
                    ship_type = EXCLUDED.ship_type
            "#,
            item.shipyard_id,
            item.ship_type as models::ShipType
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[ShipyardShipTypes],
    ) -> crate::Result<()> {
        let (shipyard_ids, ship_types): (Vec<i64>, Vec<models::ShipType>) =
            itertools::multiunzip(items.iter().map(|s| (s.shipyard_id, s.ship_type)));

        sqlx::query!(
            r#"
            INSERT INTO shipyard_ship_types (
                shipyard_id,
                ship_type
            )
            SELECT * FROM UNNEST(
                $1::bigint[],
                $2::ship_type[]
            )
            ON CONFLICT (id) DO UPDATE
            SET shipyard_id = EXCLUDED.shipyard_id,
                ship_type = EXCLUDED.ship_type
            "#,
            &shipyard_ids,
            &ship_types as &[models::ShipType]
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<ShipyardShipTypes>> {
        let erg = sqlx::query_as!(
            ShipyardShipTypes,
            r#"
            SELECT
                id,
                shipyard_id,
                ship_type as "ship_type: models::ShipType",
                created_at
            FROM shipyard_ship_types
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
