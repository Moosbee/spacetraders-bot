use space_traders_client::models;
use tracing::instrument;

use super::{
    run_paginated_query, DatabaseConnectorAsync, DbPool, MarketTradeGood, PaginatedQuery,
    PaginatedResult,
};

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

impl DatabaseConnectorAsync for MarketTrade {
    type ID = (
        String,
        models::TradeSymbol,
        sqlx::types::chrono::DateTime<chrono::Utc>,
    );

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &DbPool, item: &MarketTrade) -> crate::Result<Self::ID> {
        struct Inserted {
            created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
        }

        let inserted = sqlx::query_as!(
            Inserted,
            r#"
                INSERT INTO market_trade (waypoint_symbol, symbol, type)
                VALUES ($1, $2, $3)
                RETURNING created_at
            "#,
            item.waypoint_symbol,
            item.symbol as models::TradeSymbol,
            item.r#type as models::market_trade_good::Type
        )
        .fetch_one(&database_pool.database_pool)
        .await?;

        Ok((
            item.waypoint_symbol.clone(),
            item.symbol,
            inserted.created_at,
        ))
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn upsert(database_pool: &DbPool, item: &MarketTrade) -> crate::Result<()> {
        let _ = Self::insert_new(database_pool, item).await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &MarketTrade) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
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
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTrade>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTrade,
                    r#"
                        SELECT 
                            waypoint_symbol,
                            symbol as "symbol: models::TradeSymbol",
                            "type" as "type: models::market_trade_good::Type",
                            created_at
                        FROM market_trade
                        ORDER BY created_at DESC, waypoint_symbol ASC, symbol ASC
                        LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    MarketTrade,
                    r#"
                        SELECT 
                            waypoint_symbol,
                            symbol as "symbol: models::TradeSymbol",
                            "type" as "type: models::market_trade_good::Type",
                            created_at
                        FROM market_trade
                        ORDER BY created_at DESC, waypoint_symbol ASC, symbol ASC
                    "#
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                        SELECT COUNT(*) as "count!"
                        FROM market_trade
                    "#
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<Option<Self>> {
        let item = sqlx::query_as!(
            MarketTrade,
            r#"
                SELECT
                    waypoint_symbol,
                    symbol as "symbol: models::TradeSymbol",
                    "type" as "type: models::market_trade_good::Type",
                    created_at
                FROM market_trade
                WHERE waypoint_symbol = $1 AND symbol = $2 AND created_at = $3
                LIMIT 1
            "#,
            &id.0,
            &id.1 as &models::TradeSymbol,
            &id.2
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(item)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM market_trade
                WHERE waypoint_symbol = $1 AND symbol = $2 AND created_at = $3
            "#,
            &id.0,
            &id.1 as &models::TradeSymbol,
            &id.2
        )
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
    }

    fn set_id(&mut self, id: Self::ID) {
        self.waypoint_symbol = id.0;
        self.symbol = id.1;
        self.created_at = id.2;
    }
}

impl MarketTrade {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTrade>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTrade,
                    r#"
                    SELECT DISTINCT ON (waypoint_symbol, symbol)
                    waypoint_symbol, 
                    symbol as "symbol: models::TradeSymbol",
                    "type" as "type: models::market_trade_good::Type",
                    created_at
                    FROM public.market_trade WHERE symbol = $1
                    ORDER BY waypoint_symbol, symbol, created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    *trade_symbol as models::TradeSymbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
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
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM (
                        SELECT DISTINCT ON (waypoint_symbol, symbol)
                            waypoint_symbol,
                            symbol
                        FROM public.market_trade
                        WHERE symbol = $1
                        ORDER BY waypoint_symbol, symbol, created_at DESC
                    ) sub
                    "#,
                    *trade_symbol as models::TradeSymbol
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTrade>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTrade,
                    r#"
                    SELECT DISTINCT ON (waypoint_symbol, symbol)
                    waypoint_symbol, 
                    symbol as "symbol: models::TradeSymbol",
                    "type" as "type: models::market_trade_good::Type",
                    created_at
                    FROM public.market_trade WHERE waypoint_symbol = $1
                    ORDER BY waypoint_symbol, symbol, created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    waypoint_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
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
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM (
                        SELECT DISTINCT ON (waypoint_symbol, symbol)
                            waypoint_symbol,
                            symbol
                        FROM public.market_trade
                        WHERE waypoint_symbol = $1
                        ORDER BY waypoint_symbol, symbol, created_at DESC
                    ) sub
                    "#,
                    waypoint_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTrade>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTrade,
                    r#"
                    SELECT DISTINCT ON (waypoint_symbol, symbol)
                    waypoint_symbol, 
                    symbol as "symbol: models::TradeSymbol",
                    "type" as "type: models::market_trade_good::Type",
                    created_at
                    FROM public.market_trade
                    ORDER BY waypoint_symbol, symbol, created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
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
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM (
                        SELECT DISTINCT ON (waypoint_symbol, symbol)
                            waypoint_symbol,
                            symbol
                        FROM public.market_trade
                        ORDER BY waypoint_symbol, symbol, created_at DESC
                    ) sub
                    "#
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_system(
        database_pool: &DbPool,
        system_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTrade>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTrade,
                    r#"
                    SELECT DISTINCT ON (waypoint_symbol, market_trade.symbol)
                      market_trade.waypoint_symbol, 
                      market_trade.symbol as "symbol: models::TradeSymbol",
                      market_trade."type" as "type: models::market_trade_good::Type",
                      market_trade.created_at
                    FROM public.market_trade left join public.waypoint ON waypoint.symbol = market_trade.waypoint_symbol
                    WHERE waypoint.system_symbol = $1
                    ORDER BY waypoint_symbol, market_trade.symbol, market_trade.created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    system_symbol,
                    page_size,
                    offset
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
                    MarketTrade,
                    r#"
                    SELECT DISTINCT ON (waypoint_symbol, market_trade.symbol)
                      market_trade.waypoint_symbol, 
                      market_trade.symbol as "symbol: models::TradeSymbol",
                      market_trade."type" as "type: models::market_trade_good::Type",
                      market_trade.created_at
                    FROM public.market_trade left join public.waypoint ON waypoint.symbol = market_trade.waypoint_symbol
                    WHERE waypoint.system_symbol = $1
                    ORDER BY waypoint_symbol, market_trade.symbol, market_trade.created_at DESC
                    "#,
                    system_symbol
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM (
                        SELECT DISTINCT ON (waypoint_symbol, market_trade.symbol)
                            market_trade.waypoint_symbol,
                            market_trade.symbol
                        FROM public.market_trade left join public.waypoint ON waypoint.symbol = market_trade.waypoint_symbol
                        WHERE waypoint.system_symbol = $1
                        ORDER BY waypoint_symbol, market_trade.symbol, market_trade.created_at DESC
                    ) sub
                    "#,
                    system_symbol
                )
                .fetch_one(database_pool.get_cache_pool())
                .await?;
                Ok(count.count)
            },
        )
        .await
    }

    pub async fn get_history_by_waypoint_and_trade_symbol(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        trade_symbol: &models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTrade>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTrade,
                    r#"
                    SELECT
                    waypoint_symbol, 
                    symbol as "symbol: models::TradeSymbol",
                    "type" as "type: models::market_trade_good::Type",
                    created_at
                    FROM public.market_trade WHERE waypoint_symbol = $1 AND symbol = $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    waypoint_symbol,
                    *trade_symbol as models::TradeSymbol,
                    page_size,
                    offset
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let items = sqlx::query_as!(
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
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM public.market_trade
                    WHERE waypoint_symbol = $1 AND symbol = $2
                    "#,
                    waypoint_symbol,
                    *trade_symbol as models::TradeSymbol
                )
                .fetch_one(&database_pool.database_pool)
                .await?;
                Ok(count.count)
            },
        )
        .await
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
