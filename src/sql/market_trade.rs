use space_traders_client::models;

use super::{
    sql_models::{DatabaseConnector, MarketTrade},
    DbPool,
};

impl DatabaseConnector<MarketTrade> for MarketTrade {
    async fn insert(database_pool: &DbPool, item: &MarketTrade) -> sqlx::Result<()> {
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

    async fn insert_bulk(database_pool: &DbPool, items: &Vec<MarketTrade>) -> sqlx::Result<()> {
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

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<MarketTrade>> {
        sqlx::query_as!(
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
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl MarketTrade {
    pub async fn get_last_by_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
    ) -> sqlx::Result<Vec<MarketTrade>> {
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

    pub async fn get_last_by_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
    ) -> sqlx::Result<Vec<MarketTrade>> {
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
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(row)
    }

    pub async fn get_last(database_pool: &DbPool) -> sqlx::Result<Vec<MarketTrade>> {
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
        .fetch_all(&database_pool.database_pool)
        .await?;
        Ok(row)
    }
}
