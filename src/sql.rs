use std::{i32, str::FromStr};

use anyhow::Ok;
use log::debug;
use space_traders_client::models::{self, TradeSymbol};

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

#[derive(Clone, Default, Debug, PartialEq)]
pub struct MarketTransaction {
    /// The symbol of the waypoint.
    pub waypoint_symbol: String,
    /// The symbol of the ship that made the transaction.
    pub ship_symbol: String,
    /// The symbol of the trade good.
    pub trade_symbol: TradeSymbol,
    /// The type of transaction.
    pub r#type: models::market_transaction::Type,
    /// The number of units of the transaction.
    pub units: i32,
    /// The price per unit of the transaction.
    pub price_per_unit: i32,
    /// The total price of the transaction.
    pub total_price: i32,
    /// The timestamp of the transaction.
    pub timestamp: String,
    /// The reason for the transaction.
    /// pub reason: TransactionReason,
    pub contract: Option<String>,
}

// impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for MarketTransaction {
//     fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
//         use sqlx::Row;
//         let waypoint_symbol: String = row.try_get("waypoint_symbol")?;
//         let ship_symbol: String = row.try_get("ship_symbol")?;
//         let trade_symbol: TradeSymbol = row.try_get("trade_symbol")?;
//         let r_type: models::market_transaction::Type = row.try_get("r_type")?;
//         let units: i32 = row.try_get("units")?;
//         let price_per_unit: i32 = row.try_get("price_per_unit")?;
//         let total_price: i32 = row.try_get("total_price")?;
//         let timestamp: String = row.try_get("timestamp")?;
//         let contract: Option<String> = row.try_get("contract")?;

//         let reason = match contract {
//             Some(contract) => TransactionReason::Contract(contract),
//             None => TransactionReason::None,
//         };

//         sqlx::Result::Ok(MarketTransaction {
//             waypoint_symbol,
//             ship_symbol,
//             trade_symbol,
//             r#type: r_type,
//             units,
//             price_per_unit,
//             total_price,
//             timestamp,
//             reason,
//         })
//     }
// }

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TransactionReason {
    Contract(String),
    #[default]
    None,
}

pub trait With<T> {
    fn with(self, reason: T) -> Self;
}

impl With<TransactionReason> for MarketTransaction {
    fn with(self, reason: TransactionReason) -> Self {
        match reason {
            TransactionReason::Contract(contract) => MarketTransaction {
                contract: Some(contract),
                ..self
            },
            TransactionReason::None => MarketTransaction {
                contract: None,
                ..self
            },
        }
    }
}

impl Into<models::MarketTransaction> for MarketTransaction {
    fn into(self) -> models::MarketTransaction {
        models::MarketTransaction {
            ship_symbol: self.ship_symbol,
            trade_symbol: self.trade_symbol.to_string(),
            r#type: self.r#type,
            units: self.units,
            price_per_unit: self.price_per_unit,
            total_price: self.total_price,
            timestamp: self.timestamp,
            waypoint_symbol: self.waypoint_symbol,
        }
    }
}

impl TryFrom<models::MarketTransaction> for MarketTransaction {
    type Error = anyhow::Error;
    fn try_from(value: models::MarketTransaction) -> Result<Self, Self::Error> {
        let tr_symbol = TradeSymbol::from_str(&value.trade_symbol)?;

        Ok(MarketTransaction {
            ship_symbol: value.ship_symbol,
            trade_symbol: tr_symbol,
            r#type: value.r#type,
            units: value.units,
            price_per_unit: value.price_per_unit,
            total_price: value.total_price,
            timestamp: value.timestamp,
            waypoint_symbol: value.waypoint_symbol,
            // reason: TransactionReason::None,
            contract: None,
        })
    }
}

pub async fn insert_market_trade_good(
    database_pool: &sqlx::PgPool,
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

    let insert = insert.execute(database_pool).await.unwrap();

    debug!("Insert: {:?}", insert);
}

pub async fn get_last_waypoint_trade_goods(
    database_pool: &sqlx::PgPool,
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
    .fetch_all(database_pool)
    .await
    .unwrap();

    row
}

pub async fn get_last_market_trade_goods(database_pool: &sqlx::PgPool) -> Vec<MarketTradeGood> {
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
    .fetch_all(database_pool)
    .await
    .unwrap();

    row
}

pub async fn get_last_trade_markets(
    database_pool: &sqlx::PgPool,
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
    .fetch_all(database_pool)
    .await
    .unwrap();

    row
}

#[derive(Debug, Clone, sqlx::FromRow, PartialEq, Eq)]
pub struct MarketTrade {
    pub waypoint_symbol: String,
    pub symbol: models::TradeSymbol,
    pub created_at: sqlx::types::time::PrimitiveDateTime,
    pub r#type: models::market_trade_good::Type,
}

impl Default for MarketTrade {
    fn default() -> MarketTrade {
        MarketTrade {
            waypoint_symbol: String::new(),
            symbol: models::TradeSymbol::PreciousStones,
            created_at: sqlx::types::time::PrimitiveDateTime::MIN,
            r#type: models::market_trade_good::Type::Exchange,
        }
    }
}

pub async fn insert_market_trade(database_pool: &sqlx::PgPool, market_trades: Vec<MarketTrade>) {
    let waypoint_symbols = market_trades
        .iter()
        .map(|m| m.waypoint_symbol.clone())
        .collect::<Vec<String>>();

    let symbols = market_trades
        .iter()
        .map(|m| m.symbol)
        .collect::<Vec<models::TradeSymbol>>();
    let types = market_trades
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

    let insert = insert.execute(database_pool).await.unwrap();
    debug!("Insert: {:?}", insert);
}

pub async fn get_last_market_trades_symbol(
    database_pool: &sqlx::PgPool,
    trade_symbol: &models::TradeSymbol,
) -> Vec<MarketTrade> {
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
    .fetch_all(database_pool)
    .await
    .unwrap();
    row
}

pub async fn get_last_market_trades(database_pool: &sqlx::PgPool) -> Vec<MarketTrade> {
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
    .fetch_all(database_pool)
    .await
    .unwrap();
    row
}

pub async fn insert_waypoint(database_pool: &sqlx::PgPool, waypoints: &Vec<models::Waypoint>) {
    let (m_symbols, f_symbols): (Vec<String>, Vec<String>) = waypoints
        .iter()
        .map(|w| (w.symbol.clone(), w.system_symbol.clone()))
        .unzip();

    sqlx::query!(
        r#"
            INSERT INTO waypoint (symbol, system_symbol)
            SELECT * FROM UNNEST($1::character varying[], $2::character varying[])
            ON CONFLICT (symbol) DO UPDATE SET system_symbol = EXCLUDED.system_symbol
        "#,
        &m_symbols,
        &f_symbols
    )
    .execute(database_pool)
    .await
    .unwrap();
}

pub async fn insert_market_transaction(
    database_pool: &sqlx::PgPool,
    transaction: &MarketTransaction,
) {
    sqlx::query!(
        r#"
            INSERT INTO market_transaction (waypoint_symbol, ship_symbol, trade_symbol, "type", units, price_per_unit, total_price, "timestamp", contract)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (waypoint_symbol, ship_symbol, trade_symbol, "timestamp") DO UPDATE
            SET units = EXCLUDED.units,
            price_per_unit = EXCLUDED.price_per_unit,
            total_price = EXCLUDED.total_price
        "#,
        transaction.waypoint_symbol,
        transaction.ship_symbol,
        transaction.trade_symbol as models::TradeSymbol,
        transaction.r#type as models::market_transaction::Type,
        transaction.units,
        transaction.price_per_unit,
        transaction.total_price,
        transaction.timestamp,
        transaction.contract
    )
    .execute(database_pool)
    .await
    .unwrap();
}

pub async fn insert_market_transactions(
    database_pool: &sqlx::PgPool,
    transactions: Vec<MarketTransaction>,
) {
    let (
        ((t_waypoint_symbol, t_ship_symbol), (t_trade_symbol, t_type)),
        ((t_units, t_timestamp_and_contract), (t_price_per_unit, t_total_price)),
    ): (
        (
            (Vec<String>, Vec<String>),
            (Vec<TradeSymbol>, Vec<models::market_transaction::Type>),
        ),
        (
            (Vec<i32>, Vec<(String, Option<String>)>),
            (Vec<i32>, Vec<i32>),
        ),
    ) = transactions
        .iter()
        .map(|t| {
            (
                (
                    (t.waypoint_symbol.clone(), t.ship_symbol.clone()),
                    (t.trade_symbol.clone(), t.r#type.clone()),
                ),
                (
                    (t.units.clone(), (t.timestamp.clone(), t.contract.clone())),
                    (t.price_per_unit.clone(), t.total_price.clone()),
                ),
            )
        })
        .map(
            |f: (
                (
                    (String, String),
                    (TradeSymbol, models::market_transaction::Type),
                ),
                ((i32, (String, Option<String>)), (i32, i32)),
            )| f,
        )
        .unzip();

    let (t_timestamp, t_contract): (Vec<String>, Vec<Option<String>>) =
        t_timestamp_and_contract.into_iter().unzip();

    sqlx::query!(
        r#"
            INSERT INTO market_transaction (waypoint_symbol, ship_symbol,trade_symbol, "type", units, price_per_unit, total_price, "timestamp", contract)
              SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::trade_symbol[],
                $4::market_transaction_type[],
                $5::integer[],
                $6::integer[],
                $7::integer[],
                $8::character varying[],
                $9::character varying[]
            )
            ON CONFLICT (waypoint_symbol, ship_symbol, trade_symbol, "timestamp") DO UPDATE
            SET units = EXCLUDED.units,
            price_per_unit = EXCLUDED.price_per_unit,
            total_price = EXCLUDED.total_price
        "#,
        &t_waypoint_symbol,
        &t_ship_symbol,
        &t_trade_symbol as &[models::TradeSymbol],
        &t_type as &[models::market_transaction::Type],
        &t_units,
        &t_price_per_unit,
        &t_total_price,
        &t_timestamp,
        &t_contract as &[Option<String>]
    )
    .execute(database_pool)
    .await
    .unwrap();
}

pub async fn get_market_transactions(database_pool: &sqlx::PgPool) -> Vec<MarketTransaction> {
    let row: Vec<MarketTransaction> = sqlx::query_as!(
      MarketTransaction,
        r#"
      select waypoint_symbol, ship_symbol,trade_symbol as "trade_symbol: models::TradeSymbol", "type" as "type: models::market_transaction::Type", units, price_per_unit, total_price, "timestamp", contract from market_transaction
    "#
    ).fetch_all(database_pool).await.unwrap();
    row
}

pub async fn update_contract(database_pool: &sqlx::PgPool, contract: &models::Contract) {
    update_base_contract(database_pool, contract).await;
    if let Some(deliveries) = &contract.terms.deliver {
        update_contract_deliveries(database_pool, &contract.id, deliveries).await;
    }
}

pub async fn update_base_contract(database_pool: &sqlx::PgPool, contract: &models::Contract) {
    sqlx::query!(
        r#"
            INSERT INTO contract (
              id,
              faction_symbol,
              contract_type,
              accepted,
              fulfilled,
              deadline_to_accept,
              on_accepted,
              on_fulfilled,
              deadline
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET 
              faction_symbol = EXCLUDED.faction_symbol,
              contract_type = EXCLUDED.contract_type,
              accepted = EXCLUDED.accepted,
              fulfilled = EXCLUDED.fulfilled,
              deadline_to_accept = EXCLUDED.deadline_to_accept,
              on_accepted = EXCLUDED.on_accepted,
              on_fulfilled = EXCLUDED.on_fulfilled,
              deadline = EXCLUDED.deadline
        "#,
        contract.id,
        contract.faction_symbol,
        contract.r#type as models::contract::Type,
        contract.accepted,
        contract.fulfilled,
        contract.deadline_to_accept,
        contract.terms.payment.on_accepted,
        contract.terms.payment.on_fulfilled,
        contract.terms.deadline
    )
    .execute(database_pool)
    .await
    .unwrap();
}

pub async fn update_contract_deliveries(
    database_pool: &sqlx::PgPool,
    contract_id: &str,
    contract_deliveries: &Vec<models::ContractDeliverGood>,
) {
    let (
        ((contract_ids, trade_symbols), (units_fulfilled, units_required)),
        ((destination_symbols, _), (_, _)),
    ): (
        ((Vec<String>, Vec<TradeSymbol>), (Vec<i32>, Vec<i32>)),
        ((Vec<String>, Vec<()>), (Vec<()>, Vec<()>)),
    ) = contract_deliveries
        .iter()
        .map(|c| {
            let res = (
                (
                    (
                        contract_id.to_string(),
                        models::TradeSymbol::try_from(c.trade_symbol.as_str()).clone()?,
                    ),
                    (c.units_fulfilled.clone(), c.units_required.clone()),
                ),
                ((c.destination_symbol.clone(), ()), ((), ())),
            );
            Ok(res)
        })
        .filter_map(
            |c: Result<
                (
                    ((String, TradeSymbol), (i32, i32)),
                    ((String, ()), ((), ())),
                ),
                _,
            >| c.ok(),
        )
        .unzip();

    sqlx::query!(
        r#"
            INSERT INTO contract_delivery (
              contract_id,
              trade_symbol,
              destination_symbol,
              units_required,
              units_fulfilled
            )
              SELECT * FROM UNNEST(
                $1::character varying[],
                $2::trade_symbol[],
                $3::character varying[],
                $4::integer[],
                $5::integer[]
            )
            ON CONFLICT (contract_id, trade_symbol, destination_symbol) DO UPDATE
            SET units_fulfilled = EXCLUDED.units_fulfilled
        "#,
        &contract_ids,
        &trade_symbols as &[models::TradeSymbol],
        &destination_symbols,
        &units_required,
        &units_fulfilled
    )
    .execute(database_pool)
    .await
    .unwrap();
}
