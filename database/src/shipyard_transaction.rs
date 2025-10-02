use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::DatabaseConnector;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ShipyardTransaction {
    pub waypoint_symbol: String,
    pub ship_type: models::ShipType,
    pub price: i32,
    pub agent_symbol: String,
    pub timestamp: DateTime<Utc>,
}

impl TryFrom<models::ShipyardTransaction> for ShipyardTransaction {
    type Error = crate::Error;

    fn try_from(item: models::ShipyardTransaction) -> Result<Self, Self::Error> {
        let ship_type = models::ShipType::from_str(&item.ship_type)
            .map_err(|_| Self::Error::InvalidShipType(item.ship_type))?;
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)
            .map_err(|_| Self::Error::InvalidTimestamp(item.timestamp))?;

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
    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<ShipyardTransaction>> {
        let erg = sqlx::query_as!(
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
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_system(
        database_pool: &super::DbPool,
        system: &str,
    ) -> crate::Result<Vec<Self>> {
        let system_qr = format!("{}-%", system);
        let erg = sqlx::query_as!(
            ShipyardTransaction,
            r#"
      select 
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
      from shipyard_transaction
      where waypoint_symbol like $1
    "#,
            system_qr
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl DatabaseConnector<ShipyardTransaction> for ShipyardTransaction {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(
        database_pool: &super::DbPool,
        item: &ShipyardTransaction,
    ) -> crate::Result<()> {
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

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(
        database_pool: &super::DbPool,
        items: &[ShipyardTransaction],
    ) -> crate::Result<()> {
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
            &timestamps as &[chrono::DateTime<chrono::Utc>],
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<ShipyardTransaction>> {
        let erg = sqlx::query_as!(
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
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
