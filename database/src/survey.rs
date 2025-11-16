use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use space_traders_client::models::{self};
use tracing::instrument;

use crate::{DatabaseConnector, DbPool};

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct Survey {
    pub ship_info_before: i64,
    pub ship_info_after: i64,
    pub ship_symbol: String,
    pub signature: String,
    pub waypoint_symbol: String,
    pub deposits: Vec<models::TradeSymbol>,
    pub expiration: DateTime<Utc>,
    pub size: models::SurveySize,
    pub exhausted_since: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct SurveyPercent {
    pub symbol: models::TradeSymbol,
    pub percent: f64,
}

#[async_graphql::ComplexObject]
impl Survey {
    async fn percent(&self) -> Vec<SurveyPercent> {
        self.get_percent()
            .iter()
            .map(|f| SurveyPercent {
                symbol: f.0,
                percent: f.1,
            })
            .collect()
    }

    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<super::Waypoint>> {
        let database_pool = ctx.data::<super::DbPool>().unwrap();
        super::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await
    }

    async fn extractions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Vec<super::Extraction>> {
        let database_pool = ctx.data::<super::DbPool>().unwrap();
        super::Extraction::get_by_survey_symbol(database_pool, &self.signature).await
    }
}

impl Survey {
    pub fn from_model(
        value: models::Survey,
        ship_before: i64,
        ship_after: i64,
        ship_symbol: String,
    ) -> crate::Result<Survey> {
        let deposits = value.deposits.iter().map(|f| f.symbol).collect::<Vec<_>>();
        let expiration = DateTime::<chrono::Utc>::from_str(&value.expiration)?;

        Ok(Survey {
            ship_info_before: ship_before,
            ship_info_after: ship_after,
            ship_symbol,
            signature: value.signature,
            waypoint_symbol: value.symbol,
            deposits,
            expiration,
            size: value.size,
            exhausted_since: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn get_percent(&self) -> Vec<(models::TradeSymbol, f64)> {
        let mut items = HashMap::new();

        for item in self.deposits.iter() {
            *items.entry(*item).or_insert(0) += 1;
        }

        let mut vecs = items
            .into_iter()
            .map(|f| (f.0, f.1 as f64 / self.deposits.len() as f64))
            .collect::<Vec<_>>();

        vecs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        vecs
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_signature(
        database_pool: &DbPool,
        signature: &str,
    ) -> crate::Result<Survey> {
        let erg = sqlx::query_as!(
            Survey,
            r#"
                SELECT
                  signature,
                  ship_info_before,
                  ship_info_after,
                  ship_symbol,
                  waypoint_symbol,
                  deposits as "deposits: Vec<models::TradeSymbol>",
                  expiration,
                  size as "size: models::SurveySize",
                  exhausted_since,
                  created_at,
                  updated_at
                FROM surveys
                WHERE signature = $1
            "#,
            signature
        )
        .fetch_one(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_waypoint_symbol(
        database_pool: &DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<Survey>> {
        let erg = sqlx::query_as!(
            Survey,
            r#"
                SELECT
                  signature,
                  ship_info_before,
                  ship_info_after,
                  ship_symbol,
                  waypoint_symbol,
                  deposits as "deposits: Vec<models::TradeSymbol>",
                  expiration,
                  size as "size: models::SurveySize",
                  exhausted_since,
                  created_at,
                  updated_at
                FROM surveys
                WHERE waypoint_symbol = $1
            "#,
            waypoint_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_system_symbol(
        database_pool: &DbPool,
        system_symbol: &str,
    ) -> crate::Result<Vec<Survey>> {
        let erg = sqlx::query_as!(
            Survey,
            r#"
                SELECT
                  signature,
                  ship_info_before,
                  ship_info_after,
                  ship_symbol,
                  waypoint_symbol,
                  deposits as "deposits: Vec<models::TradeSymbol>",
                  expiration,
                  size as "size: models::SurveySize",
                  exhausted_since,
                  created_at,
                  updated_at
                FROM surveys
                WHERE waypoint_symbol IN (SELECT symbol FROM waypoint WHERE system_symbol = $1)
            "#,
            system_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_by_size(
        database_pool: &DbPool,
        size: models::SurveySize,
    ) -> crate::Result<Vec<Survey>> {
        let erg = sqlx::query_as!(
            Survey,
            r#"
                SELECT
                  signature,
                  ship_info_before,
                  ship_info_after,
                  ship_symbol,
                  waypoint_symbol,
                  deposits as "deposits: Vec<models::TradeSymbol>",
                  expiration,
                  size as "size: models::SurveySize",
                  exhausted_since,
                  created_at,
                  updated_at
                FROM surveys
                WHERE size = $1
            "#,
            size as models::SurveySize
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }

    pub async fn get_by_ship(
        database_pool: &DbPool,
        ship_symbol: &str,
    ) -> crate::Result<Vec<Survey>> {
        let erg = sqlx::query_as!(
            Survey,
            r#"
                SELECT
                  signature,
                  ship_info_before,
                  ship_info_after,
                  ship_symbol,
                  waypoint_symbol,
                  deposits as "deposits: Vec<models::TradeSymbol>",
                  expiration,
                  size as "size: models::SurveySize",
                  exhausted_since,
                  created_at,
                  updated_at
                FROM surveys
                WHERE ship_symbol = $1
            "#,
            ship_symbol
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool))]
    pub async fn get_working_for_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<Survey>> {
        let erg = sqlx::query_as!(
            Survey,
            r#"
                SELECT
                  signature,
                  ship_info_before,
                  ship_info_after,
                  ship_symbol,
                  waypoint_symbol,
                  deposits as "deposits: Vec<models::TradeSymbol>",
                  expiration,
                  size as "size: models::SurveySize",
                  exhausted_since,
                  created_at,
                  updated_at
                FROM surveys
                WHERE waypoint_symbol = $1 AND exhausted_since IS NULL AND expiration > NOW()
            "#,
            waypoint_symbol
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }
}

impl From<Survey> for models::Survey {
    fn from(value: Survey) -> Self {
        models::Survey {
            signature: value.signature,
            symbol: value.waypoint_symbol,
            deposits: value
                .deposits
                .iter()
                .map(|f| models::SurveyDeposit { symbol: *f })
                .collect(),
            expiration: value
                .expiration
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            size: value.size,
        }
    }
}

impl From<&Survey> for models::Survey {
    fn from(value: &Survey) -> Self {
        models::Survey {
            signature: value.signature.clone(),
            symbol: value.waypoint_symbol.clone(),
            deposits: value
                .deposits
                .iter()
                .map(|f| models::SurveyDeposit { symbol: *f })
                .collect(),
            expiration: value
                .expiration
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            size: value.size,
        }
    }
}

impl DatabaseConnector<Survey> for Survey {
    #[instrument(level = "trace", skip(database_pool))]
    async fn insert(database_pool: &DbPool, item: &Survey) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO surveys (
                  signature,
                  ship_info_before,
                  ship_info_after,
                  ship_symbol,
                  waypoint_symbol,
                  deposits,
                  expiration,
                  size,
                  exhausted_since,
                  updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6::trade_symbol[], $7, $8::survey_size, $9, NOW())
                ON CONFLICT (signature) DO UPDATE SET
                  ship_info_before = EXCLUDED.ship_info_before,
                  ship_info_after = EXCLUDED.ship_info_after,
                  ship_symbol = EXCLUDED.ship_symbol,
                  waypoint_symbol = EXCLUDED.waypoint_symbol,
                  deposits = EXCLUDED.deposits,
                  expiration = EXCLUDED.expiration,
                  size = EXCLUDED.size,
                  exhausted_since = EXCLUDED.exhausted_since,
                  updated_at = NOW();
            "#,
            &item.signature,
            &item.ship_info_before,
            &item.ship_info_after,
            &item.ship_symbol,
            &item.waypoint_symbol,
            &item.deposits as &[models::TradeSymbol],
            &item.expiration,
            &item.size as &models::SurveySize,
            &item.exhausted_since as &Option<DateTime<Utc>>,
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[Survey]) -> crate::Result<()> {
        for item in items {
            Self::insert(database_pool, item).await?;
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool))]
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<Survey>> {
        let erg = sqlx::query_as!(
            Survey,
            r#"
                SELECT
                  signature,
                  ship_info_before,
                  ship_info_after,
                  ship_symbol,
                  waypoint_symbol,
                  deposits as "deposits: Vec<models::TradeSymbol>",
                  expiration,
                  size as "size: models::SurveySize",
                  exhausted_since,
                  created_at,
                  updated_at
                FROM surveys
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}
