use core::hash;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use log::{debug, info};
use space_traders_client::models::{self, waypoint, WaypointTraitSymbol};
use sqlx::Execute;
use tokio::time::sleep;

use crate::api;
pub async fn scrapping_conductor(
    api: &api::Api,
    pool: &sqlx::PgPool,
    waypoints: Vec<waypoint::Waypoint>,
) {
    info!("Starting market scrapping workers");

    // sleep(Duration::from_secs(10)).await;

    let future_markets: Vec<_> = waypoints
        .iter()
        .filter(|w| {
            w.traits
                .iter()
                .any(|t| t.symbol == WaypointTraitSymbol::Marketplace)
        })
        .map(|w| {
            let api = api.clone();
            let w = w.clone();
            tokio::spawn(async move {
                debug!("Market: {:?}", w);
                api.get_market(&w.system_symbol, &w.symbol).await.unwrap()
            })
        })
        .collect();

    let mut markets = Vec::new();

    for market in future_markets {
        markets.push(market.await.unwrap().data);
    }

    info!("Markets: {:?}", markets.len());

    // let (
    //     ((m_symbol, f_symbol), (f_type, f_trade_volume)),
    //     ((f_supply, f_activity), (f_purchase_price, f_sell_price)),
    // )

    // let data: (
    //     ((String, TradeSymbol), (models::MarketTradeGood, i32)),
    //     (
    //         (models::SupplyLevel, Option<models::ActivityLevel>),
    //         (i32, i32),
    //     ),
    // )

    let (
        ((m_symbol, f_symbol), (f_type, f_trade_volume)),
        ((f_supply, f_activity), (f_purchase_price, f_sell_price)),
    ): (
        ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)),
        ((Vec<_>, Vec<_>), (Vec<_>, Vec<_>)),
    ) = markets
        .iter()
        .filter(|x| x.trade_goods.is_some())
        .map(|m| {
            m.trade_goods
                .clone()
                .unwrap()
                .iter()
                .map(|f| {
                    (
                        (
                            (m.symbol.clone(), f.symbol.clone()),
                            (f.r#type.clone(), f.trade_volume.clone()),
                        ),
                        (
                            (f.supply.clone(), f.activity.clone()),
                            (f.purchase_price.clone(), f.sell_price.clone()),
                        ),
                    )
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .map(|f| f)
        .unzip();

    debug!("Trade goods: {:?} markets: {:?}", f_symbol, m_symbol);

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
        r#"INSERT INTO market_trade_good (waypoint_symbol, symbol, type, trade_volume, supply, activity, purchase_price, sell_price)
        SELECT * FROM UNNEST($1::character varying[], $2::trade_symbol[], $3::market_trade_good_type[], $4::integer[], $5::supply_level[], $6::activity_level[], $7::integer[], $8::integer[])"#,
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
    info!("Insert: {:?}", insert);

    info!("Market scrapping workers done");
}
