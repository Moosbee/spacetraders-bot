use chrono::NaiveDateTime;

use super::DatabaseConnector;
use space_traders_client::models;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Shipyard {
    #[allow(dead_code)]
    pub id: i64,
    pub waypoint_symbol: String,
    pub modifications_fee: i32,
    #[allow(dead_code)]
    pub created_at: NaiveDateTime,
}

impl From<&models::Shipyard> for Shipyard {
    fn from(item: &models::Shipyard) -> Self {
        Self {
            id: 0,
            waypoint_symbol: item.symbol.clone(),
            modifications_fee: item.modifications_fee,
            created_at: sqlx::types::chrono::NaiveDateTime::MIN,
        }
    }
}

impl Shipyard {
    pub async fn insert_get_id(
        database_pool: &super::DbPool,
        item: &Shipyard,
    ) -> sqlx::Result<i64> {
        let id = sqlx::query!(
            r#"
              INSERT INTO shipyard (
                  waypoint_symbol,
                  modifications_fee
              )
              VALUES ($1, $2)
              RETURNING id
          "#,
            item.waypoint_symbol,
            item.modifications_fee
        )
        .fetch_one(&database_pool.database_pool)
        .await?
        .id;
        Ok(id)
    }

    pub async fn get_last_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> sqlx::Result<Option<Shipyard>> {
        sqlx::query_as!(
            Shipyard,
            r#"
            SELECT DISTINCT ON (waypoint_symbol)
                id,
                waypoint_symbol,
                modifications_fee,
                created_at
            FROM shipyard
            WHERE waypoint_symbol = $1
            ORDER BY waypoint_symbol, created_at DESC
            "#,
            waypoint_symbol
        )
        .fetch_optional(&database_pool.database_pool)
        .await
    }
}

impl DatabaseConnector<Shipyard> for Shipyard {
    async fn insert(database_pool: &super::DbPool, item: &Shipyard) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO shipyard (
                    waypoint_symbol,
                    modifications_fee
                )
                VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE
                SET waypoint_symbol = EXCLUDED.waypoint_symbol,
                    modifications_fee = EXCLUDED.modifications_fee
            "#,
            item.waypoint_symbol,
            item.modifications_fee
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(database_pool: &super::DbPool, items: &Vec<Shipyard>) -> sqlx::Result<()> {
        let (waypoint_symbols, modifications_fees): (Vec<String>, Vec<i32>) = itertools::multiunzip(
            items
                .iter()
                .map(|s| (s.waypoint_symbol.clone(), s.modifications_fee)),
        );

        sqlx::query!(
            r#"
            INSERT INTO shipyard (
                waypoint_symbol,
                modifications_fee
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::integer[]
            )
            ON CONFLICT (id) DO UPDATE
            SET waypoint_symbol = EXCLUDED.waypoint_symbol,
                modifications_fee = EXCLUDED.modifications_fee
            "#,
            &waypoint_symbols,
            &modifications_fees as &[i32],
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<Shipyard>> {
        sqlx::query_as!(
            Shipyard,
            r#"
            SELECT
                id,
                waypoint_symbol,
                modifications_fee,
                created_at
            FROM shipyard
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
