use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Connection};
use std::{path::Path, time::Duration};
use tokio::process::Command;
use tracing::{debug, error, info, instrument};

#[instrument(skip(database_url, readyset_url), fields(%database_url))]
pub async fn create_database_pool(
    database_url: &str,
    readyset_url: Option<&String>,
) -> Result<database::DbPool, anyhow::Error> {
    info!("Creating database connection pool");
    let database_pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(120))
        .connect(database_url)
        .await?;
    debug!("Database pool created successfully");

    let readyset_pool = if let Some(readyset_url) = readyset_url {
        debug!(%readyset_url, "Creating Readyset connection pool");
        Some(
            PgPoolOptions::new()
                .max_connections(20)
                .acquire_timeout(Duration::from_secs(120))
                .connect(readyset_url)
                .await?,
        )
    } else {
        None
    };

    let database_pool = database::DbPool::new(database_pool, readyset_pool);

    Ok(database_pool)
}

// destroys the current connection and completely resets the database
#[instrument(skip(db, database_url))]
pub async fn reset_database(db: database::DbPool, database_url: &str) -> Result<(), anyhow::Error> {
    info!("Resetting database");
    if let Some(readyset_pool) = &db.readyset_pool {
        readyset_pool.close().await;
    }
    let db_name = db
        .database_pool
        .connect_options()
        .get_database()
        .unwrap()
        .to_string();
    db.database_pool.close().await;

    drop(db);

    let admin_url = database_url.replace(&format!("/{db_name}"), "/postgres");
    let mut conn = sqlx::PgConnection::connect(&admin_url).await?;

    // Terminate other connections so DROP doesn't fail
    sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
    ))
    .execute(&mut conn)
    .await?;

    sqlx::query(&format!("DROP DATABASE IF EXISTS \"{db_name}\""))
        .execute(&mut conn)
        .await?;

    sqlx::query(&format!("CREATE DATABASE \"{db_name}\""))
        .execute(&mut conn)
        .await?;

    info!(db_name = %db_name, "Database reset completed");
    Ok(())
}

#[instrument(skip(database_url))]
pub async fn export_database(database_url: &str, file_name: &str) -> Result<String> {
    if !cfg!(target_os = "linux") {
        tracing::error!("Only Linux is supported for database backups");
        return Err(anyhow::anyhow!(
            "Only Linux is supported for database backups"
        ));
    }

    let dump_file = format!("./db_backup/{}.sql.gz", file_name);

    debug!("Dumping PostgreSQL database to {}", dump_file);

    // Create output directory if needed
    if let Some(parent) = Path::new(&dump_file).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Run pg_dump with gzip compression
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("pg_dump {} | gzip > {}", database_url, dump_file))
        .output()
        .await?;

    if output.status.success() {
        info!("Database dump completed: {}", dump_file);
        Ok(dump_file)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("pg_dump failed: {}", stderr);
        Err(anyhow::anyhow!("pg_dump failed: {}", stderr))
    }
}
