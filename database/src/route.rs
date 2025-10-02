use chrono::{DateTime, Utc};
use tracing::instrument;

use super::DatabaseConnector;

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub id: i32,
    pub ship_symbol: String,
    pub from: String,
    pub to: String,
    pub nav_mode: String,
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: f64,
    pub ship_info_before: Option<i64>,
    pub ship_info_after: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl DatabaseConnector<Route> for Route {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &super::DbPool, item: &Route) -> crate::Result<()> {
        sqlx::query!(
            r#"
            insert into route (
            ship_symbol,
            "from",
            "to",
            distance,
            nav_mode,
            fuel_cost,
            travel_time,
            ship_info_before,
            ship_info_after
            )
            values (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9
            )
            on conflict (id) do nothing
            "#,
            item.ship_symbol,
            item.from,
            item.to,
            item.distance,
            item.nav_mode,
            item.fuel_cost,
            item.travel_time,
            item.ship_info_before,
            item.ship_info_after
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &super::DbPool, items: &[Route]) -> crate::Result<()> {
        for item in items {
            Self::insert(database_pool, item).await?;
        }

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<Route>> {
        let erg = sqlx::query_as!(
            Route,
            r#"
                SELECT 
                  id,
                  ship_symbol,
                  "from",
                  "to",
                  nav_mode,
                  distance,
                  fuel_cost,
                  travel_time,
                  ship_info_before,
                  ship_info_after,
                  created_at
                FROM route
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
