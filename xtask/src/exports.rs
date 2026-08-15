use std::{env, time::Duration};

use database::PaginatedQuery;
use sqlx::postgres::PgPoolOptions;

pub async fn export_system_waypoints(system: &str) -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL").unwrap();

    let database_pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(120))
        .connect(&database_url)
        .await?;

    let database_pool = database::DbPool::new(database_pool, None);

    let system_waypoints =
        database::Waypoint::get_by_system(&database_pool, system, PaginatedQuery::unpaged())
            .await?;

    let waypoints_str = serde_json::to_string(&system_waypoints.items).unwrap();

    println!("{}", waypoints_str);

    Ok(())
}
