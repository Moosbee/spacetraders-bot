use log::debug;
use space_traders_client::models;

#[derive(Debug, Clone, sqlx::FromRow, PartialEq, Eq)]
pub struct MarketTradeGood {
    pub symbol: models::TradeSymbol,
    pub waypoint_symbol: String,
    pub r#type: models::market_trade_good::Type,
    pub trade_volume: i32,
    pub supply: models::SupplyLevel,
    pub activity: Option<models::ActivityLevel>,
    pub purchase_price: i32,
    pub sell_price: i32,
    pub created: sqlx::types::time::PrimitiveDateTime,
    pub created_at: sqlx::types::time::PrimitiveDateTime,
}

impl Into<models::MarketTradeGood> for MarketTradeGood {
    fn into(self) -> models::MarketTradeGood {
        models::MarketTradeGood {
            activity: self.activity,
            purchase_price: self.purchase_price,
            sell_price: self.sell_price,
            supply: self.supply,
            symbol: self.symbol,
            trade_volume: self.trade_volume,
            r#type: self.r#type,
        }
    }
}

pub async fn insert_market_trade_good(
    pool: &sqlx::PgPool,
    trade_goods: Vec<(String, models::MarketTradeGood)>,
) {
    let (
        ((m_symbol, f_symbol), (f_type, f_trade_volume)),
        ((f_supply, f_activity), (f_purchase_price, f_sell_price)),
    ): (
        ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)),
        ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)),
    ) = trade_goods
        .iter()
        .map(|m| {
            {
                (
                    (
                        (m.0.clone(), m.1.symbol.clone()),
                        (m.1.r#type.clone(), m.1.trade_volume.clone()),
                    ),
                    (
                        (m.1.supply.clone(), m.1.activity.clone()),
                        (m.1.purchase_price.clone(), m.1.sell_price.clone()),
                    ),
                )
            }
        })
        .unzip();

    // let insert = sqlx::query!(
    //     r#"INSERT INTO market_trade_good (waypoint_symbol, symbol, type, trade_volume, supply, activity, purchase_price, sell_price) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
    //     m_symbol[0],
    //     f_symbol[0] as models::TradeSymbol,
    //     f_type[0] as models::market_trade_good::Type,
    //     f_trade_volume[0],
    //     f_supply[0] as models::SupplyLevel,
    //     f_activity[0] as Option<models::ActivityLevel>,
    //     f_purchase_price[0],
    //     f_sell_price[0],
    // );

    // let mut hasher = HashSet::new();

    // m_symbol.iter().zip(f_symbol.iter()).for_each(|(m, f)| {
    //     debug!("Market: {:?} Trade good: {:?}", m, f);
    //     if hasher.contains(&(m, f)) {
    //         panic!("Market: {:?} Trade good: {:?} already exists", m, f);
    //     }
    //     hasher.insert((m, f));
    // });

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

    let insert = insert.execute(pool).await.unwrap();
    debug!("Insert: {:?}", insert);
}

pub async fn get_last_waypoint_trade_goods(
    pool: &sqlx::PgPool,
    waypoint_symbol: &str,
) -> Vec<MarketTradeGood> {
    let row = sqlx::query_as!(
        crate::sql::MarketTradeGood,
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
    .fetch_all(pool)
    .await
    .unwrap();

    row
}

pub async fn get_last_market_trade_goods(pool: &sqlx::PgPool) -> Vec<MarketTradeGood> {
    let row = sqlx::query_as!(
        crate::sql::MarketTradeGood,
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
    .fetch_all(pool)
    .await
    .unwrap();

    row
}

pub async fn get_last_trade_markets(
    pool: &sqlx::PgPool,
    trade_symbol: &models::TradeSymbol,
) -> Vec<MarketTradeGood> {
    let row = sqlx::query_as!(
        crate::sql::MarketTradeGood,
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
    .fetch_all(pool)
    .await
    .unwrap();

    row
}

pub async fn insert_waypoint(pool: &sqlx::PgPool, waypoints: &Vec<models::Waypoint>) {
    let (m_symbols, f_symbols): (Vec<String>, Vec<String>) = waypoints
        .iter()
        .map(|w| (w.symbol.clone(), w.system_symbol.clone()))
        .unzip();

    sqlx::query!(
        r#"
            INSERT INTO waypoint (symbol, system_symbol)
            SELECT * FROM UNNEST($1::character varying[], $2::character varying[])
            ON CONFLICT (symbol) DO NOTHING
        "#,
        &m_symbols,
        &f_symbols
    )
    .execute(pool)
    .await
    .unwrap();
}
