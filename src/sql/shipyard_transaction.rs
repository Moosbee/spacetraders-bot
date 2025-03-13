use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime};
use space_traders_client::models;

use super::DatabaseConnector;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ShipyardTransaction {
    pub waypoint_symbol: String,
    pub ship_type: models::ShipType,
    pub price: i32,
    pub agent_symbol: String,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid ship type: {0}")]
    ShipType(String),

    #[error("Invalid trade symbol: {0}")]
    TradeSymbol(String),

    #[error("Invalid timestamp: {0}")]
    Timestamp(String),
}

impl TryFrom<models::ShipyardTransaction> for ShipyardTransaction {
    type Error = ParseError;

    fn try_from(item: models::ShipyardTransaction) -> Result<Self, Self::Error> {
        let ship_type = models::ShipType::from_str(&item.ship_type)
            .map_err(|_| ParseError::ShipType(item.ship_type))?;
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)
            .map_err(|_| ParseError::Timestamp(item.timestamp))?
            .naive_utc();

        Ok(Self {
            waypoint_symbol: item.waypoint_symbol,
            ship_type,
            price: item.price,
            agent_symbol: item.agent_symbol,
            timestamp,
        })
    }
}

impl ShipyardTransaction {
    pub async fn get_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> sqlx::Result<Vec<ShipyardTransaction>> {
        sqlx::query_as!(
            ShipyardTransaction,
            r#"
            SELECT
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
            FROM shipyard_transaction
            WHERE waypoint_symbol = $1
            "#,
            waypoint_symbol
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl DatabaseConnector<ShipyardTransaction> for ShipyardTransaction {
    async fn insert(database_pool: &super::DbPool, item: &ShipyardTransaction) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO shipyard_transaction (
                    waypoint_symbol,
                    ship_type,
                    price,
                    agent_symbol,
                    "timestamp"
                )
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (waypoint_symbol, ship_type, agent_symbol, "timestamp") DO NOTHING
            "#,
            item.waypoint_symbol,
            item.ship_type as models::ShipType,
            item.price,
            item.agent_symbol,
            item.timestamp
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &Vec<ShipyardTransaction>,
    ) -> sqlx::Result<()> {
        let (waypoint_symbols, ship_types, prices, agent_symbols, timestamps): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|t| {
            (
                t.waypoint_symbol.clone(),
                t.ship_type,
                t.price,
                t.agent_symbol.clone(),
                t.timestamp,
            )
        }));

        sqlx::query!(
            r#"
            INSERT INTO shipyard_transaction (
                waypoint_symbol,
                ship_type,
                price,
                agent_symbol,
                "timestamp"
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::ship_type[],
                $3::integer[],
                $4::character varying[],
                $5::timestamp[]
            )
            ON CONFLICT (waypoint_symbol, ship_type, agent_symbol, "timestamp") DO NOTHING
            "#,
            &waypoint_symbols,
            &ship_types as &[models::ShipType],
            &prices,
            &agent_symbols,
            &timestamps
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    async fn get_all(database_pool: &super::DbPool) -> sqlx::Result<Vec<ShipyardTransaction>> {
        sqlx::query_as!(
            ShipyardTransaction,
            r#"
            SELECT
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
            FROM shipyard_transaction
            "#
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
