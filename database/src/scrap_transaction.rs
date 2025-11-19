use std::str::FromStr;

use chrono::{DateTime, Utc};
use space_traders_client::models;
use tracing::instrument;

use super::DatabaseConnector;

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "DBScrapTransaction")]

pub struct ScrapTransaction {
    pub id: i64,
    pub waypoint_symbol: String,
    pub ship_symbol: String,
    pub total_price: i32,
    pub timestamp: DateTime<Utc>,
}
impl TryFrom<models::ScrapTransaction> for ScrapTransaction {
    type Error = crate::Error;

    fn try_from(item: models::ScrapTransaction) -> Result<Self, Self::Error> {
        let timestamp = DateTime::<chrono::Utc>::from_str(&item.timestamp)
            .map_err(|_| Self::Error::InvalidTimestamp(item.timestamp))?;
        Ok(Self {
            id: 0,
            waypoint_symbol: item.waypoint_symbol,
            ship_symbol: item.ship_symbol,
            total_price: item.total_price,
            timestamp,
        })
    }
}

impl ScrapTransaction {
    pub async fn get_by_ship(
        database_pool: &super::DbPool,
        ship_symbol: &str,
    ) -> crate::Result<Vec<ScrapTransaction>> {
        let reg = sqlx::query_as!(
            ScrapTransaction,
            r#"
        SELECT
            id,
            waypoint_symbol,
            ship_symbol,
            total_price,
            "timestamp"
        FROM scrap_transaction
        WHERE ship_symbol = $1
        "#,
            ship_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(reg)
    }

    pub async fn get_by_waypoint(
        database_pool: &super::DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<ScrapTransaction>> {
        let reg = sqlx::query_as!(
            ScrapTransaction,
            r#"
        SELECT
            id,
            waypoint_symbol,
            ship_symbol,
            total_price,
            "timestamp"
        FROM scrap_transaction
        WHERE waypoint_symbol = $1
        "#,
            waypoint_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(reg)
    }

    pub async fn get_by_system(
        database_pool: &super::DbPool,
        symbol: &str,
    ) -> crate::Result<Vec<ScrapTransaction>> {
        let reg = sqlx::query_as!(
            ScrapTransaction,
            r#"
        SELECT
            id,
            waypoint_symbol,
            ship_symbol,
            total_price,
            "timestamp"
        FROM scrap_transaction JOIN waypoint ON scrap_transaction.waypoint_symbol = waypoint.symbol
        WHERE waypoint.system_symbol = $1
        "#,
            symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(reg)
    }
}

impl DatabaseConnector<ScrapTransaction> for ScrapTransaction {
    #[instrument(level = "trace", skip(database_pool, item))]
    async fn insert(database_pool: &super::DbPool, item: &ScrapTransaction) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO scrap_transaction (
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
        items: &[ScrapTransaction],
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
            INSERT INTO scrap_transaction (
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
            &timestamps as &[DateTime<Utc>],
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &super::DbPool) -> crate::Result<Vec<ScrapTransaction>> {
        let erg = sqlx::query_as!(
            ScrapTransaction,
            r#"
            SELECT
                id,
                waypoint_symbol,
                ship_symbol,
                total_price,
                "timestamp"
            FROM scrap_transaction
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
