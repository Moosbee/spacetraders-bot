use std::str::FromStr;

use space_traders_client::models::{self};

use super::{
    sql_models::{DatabaseConnector, MarketTransaction, TransactionReason},
    DbPool,
};

impl MarketTransaction {
    pub fn with(self, reason: TransactionReason) -> Self {
        match reason {
            TransactionReason::Contract(contract) => MarketTransaction {
                contract: Some(contract),
                trade_route: None,
                mining: None,
                ..self
            },
            TransactionReason::None => MarketTransaction {
                contract: None,
                trade_route: None,
                mining: None,
                ..self
            },
            TransactionReason::TradeRoute(route) => MarketTransaction {
                contract: None,
                trade_route: Some(route),
                mining: None,
                ..self
            },
            TransactionReason::MiningWaypoint(waypoint) => MarketTransaction {
                contract: None,
                trade_route: None,
                mining: Some(waypoint),
                ..self
            },
        }
    }
}

impl From<MarketTransaction> for models::MarketTransaction {
    fn from(val: MarketTransaction) -> Self {
        models::MarketTransaction {
            ship_symbol: val.ship_symbol,
            trade_symbol: val.trade_symbol.to_string(),
            r#type: val.r#type,
            units: val.units,
            price_per_unit: val.price_per_unit,
            total_price: val.total_price,
            timestamp: val.timestamp,
            waypoint_symbol: val.waypoint_symbol,
        }
    }
}

impl TryFrom<models::MarketTransaction> for MarketTransaction {
    type Error = anyhow::Error;
    fn try_from(value: models::MarketTransaction) -> Result<Self, Self::Error> {
        let tr_symbol = models::TradeSymbol::from_str(&value.trade_symbol)?;

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
            trade_route: None,
            mining: None,
        })
    }
}

impl DatabaseConnector<MarketTransaction> for MarketTransaction {
    async fn insert(database_pool: &DbPool, item: &MarketTransaction) -> sqlx::Result<()> {
        sqlx::query!(
        r#"
            INSERT INTO market_transaction (waypoint_symbol, ship_symbol, trade_symbol, "type", units, price_per_unit, total_price, "timestamp", contract, trade_route, mining)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (waypoint_symbol, ship_symbol, trade_symbol, "timestamp") DO UPDATE
            SET units = EXCLUDED.units,
            price_per_unit = EXCLUDED.price_per_unit,
            total_price = EXCLUDED.total_price
        "#,
        item.waypoint_symbol,
        item.ship_symbol,
        item.trade_symbol as models::TradeSymbol,
        item.r#type as models::market_transaction::Type,
        item.units,
        item.price_per_unit,
        item.total_price,
        item.timestamp,
        item.contract,
        item.trade_route,
        item.mining
    )
    .execute(&database_pool.database_pool)
    .await?;

        Ok(())
    }

    async fn insert_bulk(
        database_pool: &DbPool,
        items: &Vec<MarketTransaction>,
    ) -> sqlx::Result<()> {
        let (
            ((t_waypoint_symbol, t_ship_symbol), (t_trade_symbol, t_type)),
            (
                (t_units_and_trade_route, t_timestamp_and_contract),
                (t_price_per_unit_and_total_price, t_mining_and_klamm),
            ),
        ): (
            (
                (Vec<String>, Vec<String>),
                (
                    Vec<models::TradeSymbol>,
                    Vec<models::market_transaction::Type>,
                ),
            ),
            (
                (Vec<(i32, Option<i32>)>, Vec<(String, Option<String>)>),
                (Vec<(i32, i32)>, Vec<(Option<String>, ())>),
            ),
        ) = items
            .iter()
            .map(|t| {
                (
                    (
                        (t.waypoint_symbol.clone(), t.ship_symbol.clone()),
                        (t.trade_symbol, t.r#type),
                    ),
                    (
                        (
                            (t.units, t.trade_route),
                            (t.timestamp.clone(), t.contract.clone()),
                        ),
                        ((t.price_per_unit, t.total_price), (t.mining.clone(), ())),
                    ),
                )
            })
            .map(
                |f: (
                    (
                        (String, String),
                        (models::TradeSymbol, models::market_transaction::Type),
                    ),
                    (
                        ((i32, Option<i32>), (String, Option<String>)),
                        ((i32, i32), (Option<String>, ())),
                    ),
                )| f,
            )
            .unzip();

        let (t_timestamp, t_contract): (Vec<String>, Vec<Option<String>>) =
            t_timestamp_and_contract.into_iter().unzip();

        let (t_units, t_trade_route): (Vec<i32>, Vec<Option<i32>>) =
            t_units_and_trade_route.into_iter().unzip();

        let (t_price_per_unit, t_total_price): (Vec<i32>, Vec<i32>) =
            t_price_per_unit_and_total_price.into_iter().unzip();
        let (t_mining, _): (Vec<Option<String>>, Vec<()>) = t_mining_and_klamm.into_iter().unzip();

        sqlx::query!(
        r#"
            INSERT INTO market_transaction (waypoint_symbol, ship_symbol,trade_symbol, "type", units, price_per_unit, total_price, "timestamp", contract, trade_route, mining)
              SELECT * FROM UNNEST(
                $1::character varying[],
                $2::character varying[],
                $3::trade_symbol[],
                $4::market_transaction_type[],
                $5::integer[],
                $6::integer[],
                $7::integer[],
                $8::character varying[],
                $9::character varying[],
                $10::integer[],
                $11::character varying[]
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
        &t_contract as &[Option<String>],
        &t_trade_route as &[Option<i32>],
        &t_mining as &[Option<String>]
    )
    .execute(&database_pool.database_pool)
    .await?;

        Ok(())
    }

    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<MarketTransaction>> {
        sqlx::query_as!(
            MarketTransaction,
            r#"
      select 
        waypoint_symbol,
        ship_symbol,trade_symbol as "trade_symbol: models::TradeSymbol",
        "type" as "type: models::market_transaction::Type",
        units,
        price_per_unit,
        total_price,
        "timestamp",
        contract,
        trade_route,
        mining
      from market_transaction
    "#,
        )
        .fetch_all(&database_pool.database_pool)
        .await
    }
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
