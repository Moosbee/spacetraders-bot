use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;

use crate::{DatabaseConnector, DbPool};

#[derive(Clone, Default, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChartTransaction {
    /// The symbol of the waypoint.
    pub waypoint_symbol: String, // only one per waypoint
    /// The symbol of the ship.
    pub ship_symbol: String,
    /// The total price of the transaction.
    pub total_price: i32,
    /// The timestamp of the transaction.
    pub timestamp: DateTime<Utc>,
}

impl TryFrom<models::ChartTransaction> for ChartTransaction {
    type Error = crate::Error;
    fn try_from(item: models::ChartTransaction) -> Result<Self, Self::Error> {
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)?;

        Ok(Self {
            waypoint_symbol: item.waypoint_symbol,
            ship_symbol: item.ship_symbol,
            total_price: item.total_price,
            timestamp,
        })
    }
}

impl DatabaseConnector<ChartTransaction> for ChartTransaction {
    async fn insert(database_pool: &DbPool, item: &ChartTransaction) -> crate::Result<()> {
        sqlx::query!(
            r#"
              INSERT INTO chart_transaction (
                waypoint_symbol,
                ship_symbol,
                total_price,
                "timestamp"
              )
              VALUES ($1, $2, $3, $4)
              ON CONFLICT (waypoint_symbol) DO UPDATE SET
                ship_symbol = EXCLUDED.ship_symbol,
                total_price = EXCLUDED.total_price,
                "timestamp" = EXCLUDED."timestamp";
          "#,
            &item.waypoint_symbol,
            &item.ship_symbol,
            &item.total_price,
            &item.timestamp
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(database_pool: &DbPool, items: &[ChartTransaction]) -> crate::Result<()> {
        let (waypoint_symbols, ship_symbols, total_prices, timestamps): (
            Vec<String>,
            Vec<String>,
            Vec<i32>,
            Vec<DateTime<Utc>>,
        ) = itertools::multiunzip(items.iter().map(|ct| {
            (
                ct.waypoint_symbol.clone(),
                ct.ship_symbol.clone(),
                ct.total_price,
                ct.timestamp,
            )
        }));

        sqlx::query!(
            r#"
          INSERT INTO chart_transaction (
              waypoint_symbol,
              ship_symbol,
              total_price,
              "timestamp"
          )
          SELECT waypoint, ship, price, ts FROM UNNEST(
              $1::character varying[],
              $2::character varying[],
              $3::integer[],
              $4::timestamp with time zone[]
          ) AS t(waypoint, ship, price, ts)
          ON CONFLICT (waypoint_symbol) DO UPDATE
          SET ship_symbol = EXCLUDED.ship_symbol,
              total_price = EXCLUDED.total_price,
              "timestamp" = EXCLUDED."timestamp";
          "#,
            &waypoint_symbols,
            &ship_symbols,
            &total_prices,
            &timestamps
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<ChartTransaction>> {
        let erg = sqlx::query_as!(
            ChartTransaction,
            r#"
              SELECT
                waypoint_symbol,
                ship_symbol,
                total_price,
                "timestamp"
              FROM chart_transaction
          "#
        )
        .fetch_all(&database_pool.database_pool)
        .await?;

        Ok(erg)
    }
}
