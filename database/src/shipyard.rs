use super::DatabaseConnector;
use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBShipyard")]

pub struct Shipyard {
    #[allow(dead_code)]
    pub id: i64,
    pub waypoint_symbol: String,
    pub modifications_fee: i32,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl From<&models::Shipyard> for Shipyard {
    fn from(item: &models::Shipyard) -> Self {
        Self {
            id: 0,
            waypoint_symbol: item.symbol.clone(),
            modifications_fee: item.modifications_fee,
            created_at: sqlx::types::chrono::DateTime::<Utc>::MIN_UTC,
        }
    }
}

impl Shipyard {
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn insert_get_id(
        database_pool: &super::DbPool,
        item: &Shipyard,
    ) -> crate::Result<i64> {
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

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_last_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Option<Shipyard>> {
        let erg = sqlx::query_as!(
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
        .fetch_optional(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_last(database_pool: &super::DbPool) -> crate::Result<Vec<Shipyard>> {
        let erg = sqlx::query_as!(
            Shipyard,
            r#"
            SELECT DISTINCT ON (waypoint_symbol)
                id,
                waypoint_symbol,
                modifications_fee,
                created_at
            FROM shipyard
            ORDER BY waypoint_symbol, created_at DESC
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    pub async fn get_history_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<Shipyard>> {
        let erg = sqlx::query_as!(
            Shipyard,
            r#"
            SELECT
                id,
                waypoint_symbol,
                modifications_fee,
                created_at
            FROM shipyard
            WHERE waypoint_symbol = $1
            ORDER BY created_at DESC
            "#,
            waypoint_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnector<Shipyard> for Shipyard {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &super::DbPool, item: &Shipyard) -> crate::Result<()> {
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

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[Shipyard]) -> crate::Result<()> {
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

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<Shipyard>> {
        let erg = sqlx::query_as!(
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
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
