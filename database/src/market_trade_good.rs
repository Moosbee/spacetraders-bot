use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnectorAsync, DbPool, PaginatedQuery, PaginatedResult, run_paginated_query};

#[derive(
    Debug, Clone, sqlx::FromRow, PartialEq, Eq, serde::Serialize, async_graphql::SimpleObject,
)]
#[graphql(name = "DBMarketTradeGood")]
pub struct MarketTradeGood {
    pub symbol: models::TradeSymbol,
    pub waypoint_symbol: String,
    pub r#type: models::market_trade_good::Type,
    pub trade_volume: i32,
    pub supply: models::SupplyLevel,
    pub activity: Option<models::ActivityLevel>,
    pub purchase_price: i32,
    pub sell_price: i32,
    pub created: sqlx::types::chrono::DateTime<chrono::Utc>,
    pub created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
}

impl From<MarketTradeGood> for models::MarketTradeGood {
    fn from(val: MarketTradeGood) -> Self {
        models::MarketTradeGood {
            activity: val.activity,
            purchase_price: val.purchase_price,
            sell_price: val.sell_price,
            supply: val.supply,
            symbol: val.symbol,
            trade_volume: val.trade_volume,
            r#type: val.r#type,
        }
    }
}

impl MarketTradeGood {
    pub fn from(value: models::MarketTradeGood, waypoint_symbol: &str) -> Self {
        MarketTradeGood {
            activity: value.activity,
            purchase_price: value.purchase_price,
            sell_price: value.sell_price,
            supply: value.supply,
            symbol: value.symbol,
            trade_volume: value.trade_volume,
            r#type: value.r#type,
            waypoint_symbol: waypoint_symbol.to_string(),
            created: sqlx::types::chrono::DateTime::<chrono::Utc>::MIN_UTC, // will be ignored for inserts
            created_at: sqlx::types::chrono::DateTime::<chrono::Utc>::MIN_UTC, // will be ignored for inserts
        }
    }
}

impl DatabaseConnectorAsync for MarketTradeGood {
    type ID = (
        String,
        models::TradeSymbol,
        sqlx::types::chrono::DateTime<chrono::Utc>,
    );

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert_new(database_pool: &DbPool, item: &MarketTradeGood) -> crate::Result<Self::ID> {
        struct Inserted {
            created_at: sqlx::types::chrono::DateTime<chrono::Utc>,
        }

        let inserted = sqlx::query_as!(
            Inserted,
            r#"
                INSERT INTO market_trade_good (
                waypoint_symbol,
                symbol,
                type,
                trade_volume,
                supply,
                activity,
                purchase_price,
                sell_price
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING created_at
            "#,
            item.waypoint_symbol,
            item.symbol as models::TradeSymbol,
            item.r#type as models::market_trade_good::Type,
            item.trade_volume,
            item.supply as models::SupplyLevel,
            item.activity as Option<models::ActivityLevel>,
            item.purchase_price,
            item.sell_price
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
    async fn upsert(database_pool: &DbPool, item: &MarketTradeGood) -> crate::Result<()> {
        let _ = Self::insert_new(database_pool, item).await?;
        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn update(database_pool: &DbPool, item: &MarketTradeGood) -> crate::Result<()> {
        Self::upsert(database_pool, item).await
    }

    #[instrument(level = "trace", skip(database_pool, items))]
    async fn insert_bulk(database_pool: &DbPool, items: &[MarketTradeGood]) -> crate::Result<()> {
        let (
            m_symbol,
            f_symbol,
            f_type,
            f_trade_volume,
            f_supply,
            f_activity,
            f_purchase_price,
            f_sell_price,
        ): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = itertools::multiunzip(items.iter().map(|m| {
            (
                m.waypoint_symbol.clone(),
                m.symbol,
                m.r#type,
                m.trade_volume,
                m.supply,
                m.activity,
                m.purchase_price,
                m.sell_price,
            )
        }));

        let insert = sqlx::query!(
            r#"
            INSERT INTO market_trade_good (
                waypoint_symbol,
                symbol,
                type,
                trade_volume,
                supply,
                activity,
                purchase_price,
                sell_price
            )
            SELECT * FROM UNNEST(
                $1::character varying[],
                $2::trade_symbol[],
                $3::market_trade_good_type[],
                $4::integer[],
                $5::supply_level[],
                $6::activity_level[],
                $7::integer[],
                $8::integer[]
            )
        "#,
            &m_symbol,
            &f_symbol as &[models::TradeSymbol],
            &f_type as &[models::market_trade_good::Type],
            &f_trade_volume,
            &f_supply as &[models::SupplyLevel],
            &f_activity as &[Option<models::ActivityLevel>],
            &f_purchase_price,
            &f_sell_price,
        );

        let _insert = insert.execute(&database_pool.database_pool).await.unwrap();

        Ok(())
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTradeGood>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTradeGood,
                    r#"
                    SELECT DISTINCT ON (symbol)
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    ORDER BY symbol, created DESC
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
                    MarketTradeGood,
                    r#"
                    SELECT DISTINCT ON (symbol)
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    ORDER BY symbol, created DESC
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
                        SELECT DISTINCT ON (symbol)
                            symbol
                        FROM public.market_trade_good
                        ORDER BY symbol, created DESC
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
    async fn get_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<Option<Self>> {
        let item = sqlx::query_as!(
            MarketTradeGood,
            r#"
            SELECT
                created_at,
                created,
                waypoint_symbol,
                symbol as "symbol: models::TradeSymbol",
                "type" as "type: models::market_trade_good::Type",
                trade_volume,
                supply as "supply: models::SupplyLevel",
                activity as "activity: models::ActivityLevel",
                purchase_price,
                sell_price
            FROM public.market_trade_good
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
            DELETE FROM public.market_trade_good
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

impl MarketTradeGood {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTradeGood>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTradeGood,
                    r#"
                    SELECT
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    WHERE waypoint_symbol = $1
                    ORDER BY created DESC
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
                    MarketTradeGood,
                    r#"
                    SELECT
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    WHERE waypoint_symbol = $1
                    ORDER BY created DESC
                "#,
                    waypoint_symbol,
                )
                .fetch_all(database_pool.get_cache_pool())
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM public.market_trade_good
                    WHERE waypoint_symbol = $1
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
    pub async fn get_history_by_waypoint_and_trade_symbol(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        trade_symbol: &models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTradeGood>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTradeGood,
                    r#"
                    SELECT
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    WHERE waypoint_symbol = $1 AND symbol = $2
                    ORDER BY created DESC
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
                    MarketTradeGood,
                    r#"
                    SELECT
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    WHERE waypoint_symbol = $1 AND symbol = $2
                    ORDER BY created DESC
                "#,
                    waypoint_symbol,
                    *trade_symbol as models::TradeSymbol,
                )
                .fetch_all(&database_pool.database_pool)
                .await?;
                Ok(items)
            },
            || async move {
                let count = sqlx::query!(
                    r#"
                    SELECT COUNT(*) as "count!"
                    FROM public.market_trade_good
                    WHERE waypoint_symbol = $1 AND symbol = $2
                    "#,
                    waypoint_symbol,
                    *trade_symbol as models::TradeSymbol,
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
    ) -> crate::Result<PaginatedResult<MarketTradeGood>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTradeGood,
                    r#"
                    SELECT DISTINCT ON (symbol)
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    WHERE waypoint_symbol = $1
                    ORDER BY symbol, created DESC
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
                    MarketTradeGood,
                    r#"
                    SELECT DISTINCT ON (symbol)
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    WHERE waypoint_symbol = $1
                    ORDER BY symbol, created DESC
                "#,
                    waypoint_symbol,
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
                        SELECT DISTINCT ON (symbol)
                            symbol
                        FROM public.market_trade_good
                        WHERE waypoint_symbol = $1
                        ORDER BY symbol, created DESC
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
    pub async fn get_last_by_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTradeGood>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTradeGood,
                    r#"
                SELECT DISTINCT ON (waypoint_symbol)
                    created_at,
                    created,
                    waypoint_symbol,
                    symbol as "symbol: models::TradeSymbol",
                    "type" as "type: models::market_trade_good::Type",
                    trade_volume,
                    supply as "supply: models::SupplyLevel",
                    activity as "activity: models::ActivityLevel",
                    purchase_price,
                    sell_price
                FROM public.market_trade_good
                WHERE symbol = $1::trade_symbol
                ORDER BY waypoint_symbol, created DESC
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
                    MarketTradeGood,
                    r#"
                SELECT DISTINCT ON (waypoint_symbol)
                    created_at,
                    created,
                    waypoint_symbol,
                    symbol as "symbol: models::TradeSymbol",
                    "type" as "type: models::market_trade_good::Type",
                    trade_volume,
                    supply as "supply: models::SupplyLevel",
                    activity as "activity: models::ActivityLevel",
                    purchase_price,
                    sell_price
                FROM public.market_trade_good
                WHERE symbol = $1::trade_symbol
                ORDER BY waypoint_symbol, created DESC
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
                        SELECT DISTINCT ON (waypoint_symbol)
                            waypoint_symbol
                        FROM public.market_trade_good
                        WHERE symbol = $1::trade_symbol
                        ORDER BY waypoint_symbol, created DESC
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

    pub async fn get_by_last_waypoint_and_trade_symbol(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        trade_symbol: &models::TradeSymbol,
    ) -> crate::Result<Option<MarketTradeGood>> {
        let erg = sqlx::query_as!(
            MarketTradeGood,
            r#"
            SELECT
                created_at,
                created,
                waypoint_symbol,
                symbol as "symbol: models::TradeSymbol",
                "type" as "type: models::market_trade_good::Type",
                trade_volume,
                supply as "supply: models::SupplyLevel",
                activity as "activity: models::ActivityLevel",
                purchase_price,
                sell_price
            FROM public.market_trade_good
            WHERE waypoint_symbol = $1 AND symbol = $2
            ORDER BY created DESC
            LIMIT 1
        "#,
            waypoint_symbol,
            *trade_symbol as models::TradeSymbol,
        )
        .fetch_optional(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<MarketTradeGood>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTradeGood,
                    r#"
                    SELECT DISTINCT ON (symbol, waypoint_symbol)
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    ORDER BY symbol, waypoint_symbol, created DESC
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
                    MarketTradeGood,
                    r#"
                    SELECT DISTINCT ON (symbol, waypoint_symbol)
                        created_at,
                        created,
                        waypoint_symbol,
                        symbol as "symbol: models::TradeSymbol",
                        "type" as "type: models::market_trade_good::Type",
                        trade_volume,
                        supply as "supply: models::SupplyLevel",
                        activity as "activity: models::ActivityLevel",
                        purchase_price,
                        sell_price
                    FROM public.market_trade_good
                    ORDER BY symbol, waypoint_symbol, created DESC
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
                        SELECT DISTINCT ON (symbol, waypoint_symbol)
                            symbol,
                            waypoint_symbol
                        FROM public.market_trade_good
                        ORDER BY symbol, waypoint_symbol, created DESC
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
    ) -> crate::Result<PaginatedResult<MarketTradeGood>> {
        run_paginated_query(
            query,
            |page_size, offset| async move {
                let items = sqlx::query_as!(
                    MarketTradeGood,
                    r#"
                SELECT DISTINCT ON (waypoint_symbol, market_trade_good.symbol)
                    market_trade_good.created_at,
                    market_trade_good.created,
                    market_trade_good.waypoint_symbol,
                    market_trade_good.symbol as "symbol: models::TradeSymbol",
                    market_trade_good."type" as "type: models::market_trade_good::Type",
                    market_trade_good.trade_volume,
                    market_trade_good.supply as "supply: models::SupplyLevel",
                    market_trade_good.activity as "activity: models::ActivityLevel",
                    market_trade_good.purchase_price,
                    market_trade_good.sell_price
                FROM public.market_trade_good left join public.waypoint ON waypoint.symbol = market_trade_good.waypoint_symbol
                WHERE waypoint.system_symbol = $1
                ORDER BY waypoint_symbol, market_trade_good.symbol, created DESC
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
                    MarketTradeGood,
                    r#"
                SELECT DISTINCT ON (waypoint_symbol, market_trade_good.symbol)
                    market_trade_good.created_at,
                    market_trade_good.created,
                    market_trade_good.waypoint_symbol,
                    market_trade_good.symbol as "symbol: models::TradeSymbol",
                    market_trade_good."type" as "type: models::market_trade_good::Type",
                    market_trade_good.trade_volume,
                    market_trade_good.supply as "supply: models::SupplyLevel",
                    market_trade_good.activity as "activity: models::ActivityLevel",
                    market_trade_good.purchase_price,
                    market_trade_good.sell_price
                FROM public.market_trade_good left join public.waypoint ON waypoint.symbol = market_trade_good.waypoint_symbol
                WHERE waypoint.system_symbol = $1
                ORDER BY waypoint_symbol, market_trade_good.symbol, created DESC
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
                        SELECT DISTINCT ON (waypoint_symbol, market_trade_good.symbol)
                            market_trade_good.waypoint_symbol,
                            market_trade_good.symbol
                        FROM public.market_trade_good left join public.waypoint ON waypoint.symbol = market_trade_good.waypoint_symbol
                        WHERE waypoint.system_symbol = $1
                        ORDER BY waypoint_symbol, market_trade_good.symbol, created DESC
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
}
