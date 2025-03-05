use chrono::NaiveDateTime;

use super::DatabaseConnector;

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub id: i32,
    pub from: String,
    pub to: String,
    pub nav_mode: String,
    pub speed: i32,
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: f64,
    pub engine_condition: f64,
    pub frame_condition: f64,
    pub reactor_condition: f64,
    pub current_cargo: i32,
    pub total_cargohold: i32,
    pub ship_info_before: Option<i64>,
    pub ship_info_after: Option<i64>,
    pub created_at: NaiveDateTime,
}

impl DatabaseConnector<Route> for Route {
    async fn insert(database_pool: &super::DbPool, item: &Route) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            insert into route (
            "from",
            "to",
            distance,
            nav_mode,
            speed,
            fuel_cost,
            travel_time,
            engine_condition,
            frame_condition,
            reactor_condition,
            current_cargo,
            total_cargohold,
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
            $9,
            $10,
            $11,
            $12,
            $13,
            $14
            )
            on conflict (id) do nothing
            "#,
            item.from,
            item.to,
            item.distance,
            item.nav_mode,
            item.speed,
            item.fuel_cost,
            item.travel_time,
            item.engine_condition,
            item.frame_condition,
            item.reactor_condition,
            item.current_cargo,
            item.total_cargohold,
            item.ship_info_before,
            item.ship_info_after
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(database_pool: &super::DbPool, items: &Vec<Route>) -> sqlx::Result<()> {
        for item in items {
            Self::insert(database_pool, item).await?;
        }

        Ok(())
    }

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<Route>> {
        sqlx::query_as!(
            Route,
            r#"
                SELECT 
                  id,
                  "from",
                  "to",
                  nav_mode,
                  distance,
                  speed,
                  fuel_cost,
                  travel_time,
                  engine_condition,
                  frame_condition,
                  reactor_condition,
                  current_cargo,
                  total_cargohold,
                  ship_info_before,
                  ship_info_after,
                  created_at
                FROM route
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
