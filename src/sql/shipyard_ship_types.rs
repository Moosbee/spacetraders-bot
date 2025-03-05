use chrono::NaiveDateTime;
use space_traders_client::models;

use super::DatabaseConnector;

pub struct ShipyardShipTypes {
    #[allow(dead_code)]
    pub id: i64,
    pub shipyard_id: i64,
    pub ship_type: models::ShipType,
    #[allow(dead_code)]
    pub created_at: NaiveDateTime,
}

impl DatabaseConnector<ShipyardShipTypes> for ShipyardShipTypes {
    async fn insert(database_pool: &super::DbPool, item: &ShipyardShipTypes) -> sqlx::Result<()> {
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

    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &Vec<ShipyardShipTypes>,
    ) -> sqlx::Result<()> {
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

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<ShipyardShipTypes>> {
        sqlx::query_as!(
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
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
