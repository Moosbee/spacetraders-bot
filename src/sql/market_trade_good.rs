use space_traders_client::models;

use super::{
    sql_models::{DatabaseConnector, MarketTradeGood},
    DbPool,
};

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
            created: sqlx::types::time::PrimitiveDateTime::MIN, // will be ignored for inserts
            created_at: sqlx::types::time::PrimitiveDateTime::MIN, // will be ignored for inserts
        }
    }
}

impl DatabaseConnector<MarketTradeGood> for MarketTradeGood {
    async fn insert(database_pool: &DbPool, item: &MarketTradeGood) -> sqlx::Result<()> {
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

    async fn insert_bulk(database_pool: &DbPool, items: &Vec<MarketTradeGood>) -> sqlx::Result<()> {
        let (
            ((m_symbol, f_symbol), (f_type, f_trade_volume)),
            ((f_supply, f_activity), (f_purchase_price, f_sell_price)),
        ): (
            ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)),
            ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)),
        ) = items
            .iter()
            .map(|m| {
                {
                    (
                        (
                            (m.waypoint_symbol.clone(), m.symbol),
                            (m.r#type, m.trade_volume),
                        ),
                        ((m.supply, m.activity), (m.purchase_price, m.sell_price)),
                    )
                }
            })
            .unzip();

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

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<MarketTradeGood>> {
        sqlx::query_as!(
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
        .fetch_all(&database_pool.database_pool)
        .await
    }
}

impl MarketTradeGood {
    pub async fn get_last_by_waypoint(
        database_pool: &DbPool,
        waypoint_symbol: &str,
    ) -> sqlx::Result<Vec<MarketTradeGood>> {
        sqlx::query_as!(
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
        .fetch_all(&database_pool.database_pool)
        .await
    }

    pub async fn get_last_by_symbol(
        database_pool: &DbPool,
        trade_symbol: &models::TradeSymbol,
    ) -> sqlx::Result<Vec<MarketTradeGood>> {
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

    pub async fn get_last(database_pool: &DbPool) -> sqlx::Result<Vec<MarketTradeGood>> {
        sqlx::query_as!(
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
        .fetch_all(&database_pool.database_pool)
        .await
    }
}
