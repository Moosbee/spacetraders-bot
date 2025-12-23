use space_traders_client::models;
use tracing::instrument;

use super::{DatabaseConnector, DbPool};

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

impl DatabaseConnector<MarketTradeGood> for MarketTradeGood {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    async fn insert(database_pool: &DbPool, item: &MarketTradeGood) -> crate::Result<()> {
        sqlx::query!(
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
        .execute(&database_pool.database_pool)
        .await?;
        Ok(())
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
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<MarketTradeGood>> {
        let erg = sqlx::query_as!(
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
        Ok(erg)
    }
}

impl MarketTradeGood {
    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_by_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<MarketTradeGood>> {
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
            WHERE waypoint_symbol = $1
            ORDER BY created DESC
        "#,
            waypoint_symbol,
        )
        .fetch_all(database_pool.get_cache_pool())
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_history_by_waypoint_and_trade_symbol(
        database_pool: &DbPool,
        waypoint_symbol: &str,
        trade_symbol: &models::TradeSymbol,
    ) -> crate::Result<Vec<MarketTradeGood>> {
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
        "#,
            waypoint_symbol,
            *trade_symbol as models::TradeSymbol,
        )
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
    ) -> crate::Result<Vec<MarketTradeGood>> {
        let erg = sqlx::query_as!(
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
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
    ) -> crate::Result<Vec<MarketTradeGood>> {
        let row = sqlx::query_as!(
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

        Ok(row)
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
    pub async fn get_last(database_pool: &DbPool) -> crate::Result<Vec<MarketTradeGood>> {
        let erg = sqlx::query_as!(
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
        Ok(erg)
    }

    #[instrument(level = "trace", skip(database_pool), err(Debug))]
    pub async fn get_last_by_system(
        database_pool: &DbPool,
        system_symbol: &str,
    ) -> crate::Result<Vec<MarketTradeGood>> {
        let row = sqlx::query_as!(
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

        Ok(row)
    }
}
