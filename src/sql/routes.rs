use super::{sql_models::Route, DatabaseConnector};

impl DatabaseConnector<Route> for Route {
    async fn insert(database_pool: &super::DbPool, item: &Route) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            insert into route ( "from", "to", nav_mode, speed, fuel_cost, travel_time)
            values ($1, $2, $3, $4, $5, $6)
            on conflict (id) do nothing
            "#,
            item.from,
            item.to,
            item.nav_mode,
            item.speed,
            item.fuel_cost,
            item.travel_time
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    async fn insert_bulk(database_pool: &super::DbPool, items: &Vec<Route>) -> sqlx::Result<()> {
        let (((id_s, _), (from_s, to_s)), ((nav_mode_s, speed_s), (fuel_cost_s, travel_time_s))): (
            ((Vec<i32>, Vec<()>), (Vec<String>, Vec<String>)),
            ((Vec<String>, Vec<i32>), (Vec<i32>, Vec<f64>)),
        ) = items
            .iter()
            .cloned()
            .map(|r| {
                (
                    ((r.id, ()), (r.from, r.to)),
                    ((r.nav_mode, r.speed), (r.fuel_cost, r.travel_time)),
                )
            })
            .unzip();

        sqlx::query!(
            r#"
            insert into route (
              id,
              "from",
              "to",
              nav_mode,
              speed,
              fuel_cost,
              travel_time
            )
            SELECT * FROM UNNEST(
              $1::integer[],
              $2::character varying[],
              $3::character varying[],
              $4::character varying[],
              $5::integer[],
              $6::integer[],
              $7::double precision[]
            )
            on conflict (id) do nothing
            "#,
            &id_s,
            &from_s,
            &to_s,
            &nav_mode_s,
            &speed_s,
            &fuel_cost_s,
            &travel_time_s
        )
        .execute(&database_pool.database_pool)
        .await?;

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
                  speed,
                  fuel_cost,
                  travel_time
                FROM route
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
