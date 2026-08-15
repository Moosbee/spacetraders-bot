use std::{env, time::Duration};

use database::{DatabaseConnectorAsync, DbPool, PaginatedQuery};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

pub async fn export_waypoints(system: Option<&str>) -> anyhow::Result<()> {
    let database_pool = connect().await?;

    let waypoints = if let Some(system) = system {
        database::Waypoint::get_by_system(&database_pool, system, PaginatedQuery::unpaged()).await?
    } else {
        database::Waypoint::get_all(&database_pool, PaginatedQuery::unpaged()).await?
    };

    print_json(&waypoints.items);

    Ok(())
}

pub async fn export_systems() -> anyhow::Result<()> {
    let database_pool = connect().await?;

    let systems = database::System::get_all(&database_pool, PaginatedQuery::unpaged()).await?;

    print_json(&systems.items);

    Ok(())
}

pub async fn export_jump_connections() -> anyhow::Result<()> {
    let database_pool = connect().await?;

    let connections = ship::autopilot::generate_all_connections(&database_pool).await?;

    let output = json!({
        "connections":connections.0,
        "system_to_gate_mapping":connections.1
    });

    print_json(&output);

    Ok(())
}

pub async fn export_routes() -> anyhow::Result<()> {
    let database_pool = connect().await?;

    let routes = database::Route::get_all(&database_pool, PaginatedQuery::unpaged()).await?;

    print_json(&routes.items);

    Ok(())
}

async fn connect() -> anyhow::Result<DbPool> {
    let database_url = env::var("DATABASE_URL").unwrap();

    let database_pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(120))
        .connect(&database_url)
        .await?;

    Ok(DbPool::new(database_pool, None))
}

fn print_json<T: serde::Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}
