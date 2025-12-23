use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnector, DbPool, MarketTradeGood};

#[derive(
    Debug, Clone, sqlx::FromRow, PartialEq, Eq, serde::Serialize, async_graphql::SimpleObject,
)]
#[graphql(name = "DBMarketTrade")]
pub struct MarketTrade {
    pub waypoint_symbol: String,
    pub symbol: models::TradeSymbol,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub r#type: models::market_trade_good::Type,
}

impl Default for MarketTrade {
    fn default() -> MarketTrade {
        MarketTrade {
            waypoint_symbol: String::new(),
            symbol: models::TradeSymbol::PreciousStones,
            created_at: sqlx::types::chrono::DateTime::<chrono::Utc>::MIN_UTC,
            r#type: models::market_trade_good::Type::Exchange,
        }
    }
}

impl From<MarketTradeGood> for MarketTrade {
    fn from(value: MarketTradeGood) -> MarketTrade {
        MarketTrade {
            waypoint_symbol: value.waypoint_symbol,
            symbol: value.symbol,
            created_at: value.created_at,
            r#type: value.r#type,
        }
    }
}

impl DatabaseConnector<MarketTrade> for MarketTrade {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert(database_pool: &DbPool, item: &MarketTrade) -> crate::Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO market_trade (waypoint_symbol, symbol, type)
                VALUES ($1, $2, $3)
            "#,
            item.waypoint_symbol,
            item.symbol as models::TradeSymbol,
            item.r#type as models::market_trade_good::Type
        )
        .execute(&database_pool.database_pool)
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[MarketTrade]) -> crate::Result<()> {
        let waypoint_symbols = items
            .iter()
            .map(|m| m.waypoint_symbol.clone())
            .collect::<Vec<String>>();

        let symbols = items
            .iter()
            .map(|m| m.symbol)
            .collect::<Vec<models::TradeSymbol>>();
        let types = items
            .iter()
            .map(|m| m.r#type as models::market_trade_good::Type)
            .collect::<Vec<models::market_trade_good::Type>>();
        let insert = sqlx::query!(
            r#"
            INSERT INTO market_trade (
                waypoint_symbol,
                symbol,
                type
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::trade_symbol[],
                $3::market_trade_good_type[]
            )
        "#,
            &waypoint_symbols,
            &symbols as &[models::TradeSymbol],
            &types as &[models::market_trade_good::Type]
        );

        let _insert = insert.execute(&database_pool.database_pool).await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<MarketTrade>> {
        let erg = sqlx::query_as!(
            MarketTrade,
            r#"
                SELECT 
                    waypoint_symbol,
                    symbol as "symbol: models::TradeSymbol",
                    "type" as "type: models::market_trade_good::Type",
                    created_at
                FROM market_trade
            "#
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }
}

impl MarketTrade {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
    ) -> crate::Result<Vec<MarketTrade>> {
        let row: Vec<MarketTrade> = sqlx::query_as!(
            MarketTrade,
            r#"
            SELECT DISTINCT ON (waypoint_symbol, symbol)
            waypoint_symbol, 
            symbol as "symbol: models::TradeSymbol",
            "type" as "type: models::market_trade_good::Type",
            created_at
            FROM public.market_trade WHERE symbol = $1
            ORDER BY waypoint_symbol, symbol, created_at DESC
    "#,
            *trade_symbol as models::TradeSymbol
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(row)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<MarketTrade>> {
        let row: Vec<MarketTrade> = sqlx::query_as!(
            MarketTrade,
            r#"
            SELECT DISTINCT ON (waypoint_symbol, symbol)
            waypoint_symbol, 
            symbol as "symbol: models::TradeSymbol",
            "type" as "type: models::market_trade_good::Type",
            created_at
            FROM public.market_trade WHERE waypoint_symbol = $1
            ORDER BY waypoint_symbol, symbol, created_at DESC
    "#,
            waypoint_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(row)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last(database_pool: &DbPool) -> crate::Result<Vec<MarketTrade>> {
        let row: Vec<MarketTrade> = sqlx::query_as!(
            MarketTrade,
            r#"
            SELECT DISTINCT ON (waypoint_symbol, symbol)
            waypoint_symbol, 
            symbol as "symbol: models::TradeSymbol",
            "type" as "type: models::market_trade_good::Type",
            created_at
            FROM public.market_trade
            ORDER BY waypoint_symbol, symbol, created_at DESC
    "#,
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(row)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_system(
        database_pool: &DbPool,
        system_symbol: &str,
    ) -> crate::Result<Vec<MarketTrade>> {
        let row: Vec<MarketTrade> = sqlx::query_as!(
            MarketTrade,
            r#"
            SELECT DISTINCT ON (waypoint_symbol, market_trade.symbol)
              market_trade.waypoint_symbol, 
              market_trade.symbol as "symbol: models::TradeSymbol",
              market_trade."type" as "type: models::market_trade_good::Type",
              market_trade.created_at
            FROM public.market_trade left join public.waypoint ON waypoint.symbol = market_trade.waypoint_symbol
            WHERE waypoint.system_symbol = $1
            ORDER BY waypoint_symbol, market_trade.symbol, created_at DESC
    "#,
            system_symbol
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(row)
    }

    pub async fn get_history_by_waypoint_and_trade_symbol(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        trade_symbol: &models::TradeSymbol,
    ) -> crate::Result<Vec<MarketTrade>> {
        let row: Vec<MarketTrade> = sqlx::query_as!(
            MarketTrade,
            r#"
            SELECT
            waypoint_symbol, 
            symbol as "symbol: models::TradeSymbol",
            "type" as "type: models::market_trade_good::Type",
            created_at
            FROM public.market_trade WHERE waypoint_symbol = $1 AND symbol = $2
            ORDER BY created_at DESC
    "#,
            waypoint_symbol,
            *trade_symbol as models::TradeSymbol
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(row)
    }

    pub async fn get_last_by_waypoint_and_trade_symbol(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        trade_symbol: &models::TradeSymbol,
    ) -> crate::Result<Option<MarketTrade>> {
        let erg = sqlx::query_as!(
            MarketTrade,
            r#"
            SELECT
            waypoint_symbol, 
            symbol as "symbol: models::TradeSymbol",
            "type" as "type: models::market_trade_good::Type",
            created_at
            FROM public.market_trade WHERE waypoint_symbol = $1 AND symbol = $2
            ORDER BY created_at DESC
            LIMIT 1
    "#,
            waypoint_symbol,
            *trade_symbol as models::TradeSymbol
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }
}
