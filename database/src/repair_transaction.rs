use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::DatabaseConnector;

pub struct RepairTransaction {
    pub waypoint_symbol: String,
    pub ship_symbol: String,
    pub total_price: i32,
    pub timestamp: DateTime<Utc>,
}

impl TryFrom<models::RepairTransaction> for RepairTransaction {
    type Error = crate::Error;

    fn try_from(item: models::RepairTransaction) -> Result<Self, Self::Error> {
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)
            .map_err(|_| Self::Error::InvalidTimestamp(item.timestamp))?;
        Ok(Self {
            waypoint_symbol: item.waypoint_symbol,
            ship_symbol: item.ship_symbol,
            total_price: item.total_price,
            timestamp,
        })
    }
}

impl DatabaseConnector<RepairTransaction> for RepairTransaction {
    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert(database_pool: &super::DbPool, item: &RepairTransaction) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO repair_transaction (
                    waypoint_symbol,
                    ship_symbol,
                    total_price,
                    "timestamp"
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (waypoint_symbol, ship_symbol, "timestamp") DO NOTHING
            "#,
            item.waypoint_symbol,
            item.ship_symbol,
            item.total_price,
            item.timestamp
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[RepairTransaction],
    ) -> crate::Result<()> {
        let (waypoint_symbols, ship_symbols, total_prices, timestamps): (
            Vec<String>,
            Vec<String>,
            Vec<i32>,
            Vec<DateTime<Utc>>,
        ) = itertools::multiunzip(items.iter().map(|t| {
            (
                t.waypoint_symbol.clone(),
                t.ship_symbol.clone(),
                t.total_price,
                t.timestamp,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO repair_transaction (
                waypoint_symbol,
                ship_symbol,
                total_price,
                "timestamp"
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::integer[],
                $4::timestamp[]
            )
            ON CONFLICT (waypoint_symbol, ship_symbol, "timestamp") DO NOTHING
            "#,
            &waypoint_symbols,
            &ship_symbols,
            &total_prices,
            &timestamps as &[chrono::DateTime<Utc>],
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<RepairTransaction>> {
        let erg = sqlx::query_as!(
            RepairTransaction,
            r#"
            SELECT
                waypoint_symbol,
                ship_symbol,
                total_price,
                "timestamp"
            FROM repair_transaction
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }
}
