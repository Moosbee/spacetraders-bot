use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::DatabaseConnector;

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "DBShipyardTransaction")]
pub struct ShipyardTransaction {
    pub id: i64,
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
            id: 0,
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
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
            FROM shipyard_transaction
            WHERE waypoint_symbol = $1
            order by "timestamp"
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
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
      from shipyard_transaction
      where waypoint_symbol like $1
            order by "timestamp"
    "#,
            system_qr
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_ship_type(
        database_pool: &super::DbPool,
        ship_type: models::ShipType,
    ) -> crate::Result<Vec<ShipyardTransaction>> {
        let erg = sqlx::query_as!(
            ShipyardTransaction,
            r#"
            SELECT
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
            FROM shipyard_transaction
            WHERE ship_type = $1
            order by "timestamp"
            "#,
            ship_type as models::ShipType
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_agent(
        database_pool: &super::DbPool,
        agent_symbol: &str,
    ) -> crate::Result<Vec<ShipyardTransaction>> {
        let erg = sqlx::query_as!(
            ShipyardTransaction,
            r#"
            SELECT
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
            FROM shipyard_transaction
            WHERE agent_symbol = $1
            order by "timestamp"
            "#,
            agent_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    pub async fn get_by_id(
        database_pool: &super::DbPool,
        id: i64,
    ) -> crate::Result<ShipyardTransaction> {
        let erg = sqlx::query_as!(
            ShipyardTransaction,
            r#"
            SELECT
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
            FROM shipyard_transaction
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    pub async fn get_by_waypoint_and_ship_type(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
        ship_type: models::ShipType,
    ) -> crate::Result<Vec<ShipyardTransaction>> {
        let erg = sqlx::query_as!(
            ShipyardTransaction,
            r#"
            SELECT
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
            FROM shipyard_transaction
            WHERE waypoint_symbol = $1 AND ship_type = $2
            order by "timestamp"
            "#,
            waypoint_symbol,
            ship_type as models::ShipType
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    pub async fn insert_new(
        database_pool: &super::DbPool,
        item: &ShipyardTransaction,
    ) -> crate::Result<i64> {
        let erg = sqlx::query!(
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
                RETURNING id
            "#,
            item.waypoint_symbol,
            item.ship_type as models::ShipType,
            item.price,
            item.agent_symbol,
            item.timestamp
        )
        .fetch_one(&database_pool.database_pool)
        .await?;
        Ok(erg.id)
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
                id,
                waypoint_symbol,
                ship_type as "ship_type: models::ShipType",
                price,
                agent_symbol,
                "timestamp"
            FROM shipyard_transaction
            order by "timestamp"
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
