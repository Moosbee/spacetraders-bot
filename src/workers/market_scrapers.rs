use log::{debug, info};
use space_traders_client::models;

use crate::{
    api,
    sql::{self, insert_market_trade, insert_market_trade_good, insert_market_transactions},
};
pub async fn scrapping_conductor(
    api: api::Api,
    database_pool: sqlx::PgPool,
    waypoints: Vec<models::Waypoint>,
) {
    info!("Starting market scrapping workers");

    // sleep(Duration::from_secs(10)).await;

    let future_markets: Vec<_> = waypoints
        .iter()
        .filter(|w| {
            w.traits
                .iter()
                .any(|t| t.symbol == models::WaypointTraitSymbol::Marketplace)
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
        let market_data = market.await.unwrap().data;
        debug!("Market: {:?}", market_data.symbol);
        markets.push(market_data);
    }

    info!("Markets: {:?}", markets.len());

    insert_market_trade_good(
        &database_pool,
        markets
            .iter()
            .filter(|m| m.trade_goods.is_some())
            .map(|m| {
                m.trade_goods
                    .clone()
                    .unwrap()
                    .iter()
                    .map(|f| (m.symbol.clone(), f.clone()))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect(),
    )
    .await;
    insert_market_transactions(
        &database_pool,
        markets
            .iter()
            .filter(|m| m.transactions.is_some())
            .map(|m| m.transactions.clone().unwrap())
            .flatten()
            .map(|mt| sql::MarketTransaction::try_from(mt).unwrap())
            .collect::<Vec<_>>(),
    )
    .await;
    let market_trades: Vec<_> = markets
        .iter()
        .map(|m| {
            vec![
                m.exchange
                    .iter()
                    .map(|e| sql::MarketTrade {
                        waypoint_symbol: m.symbol.clone(),
                        symbol: e.symbol.clone(),
                        r#type: models::market_trade_good::Type::Exchange,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
                m.exports
                    .iter()
                    .map(|e| sql::MarketTrade {
                        waypoint_symbol: m.symbol.clone(),
                        symbol: e.symbol.clone(),
                        r#type: models::market_trade_good::Type::Export,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
                m.imports
                    .iter()
                    .map(|e| sql::MarketTrade {
                        waypoint_symbol: m.symbol.clone(),
                        symbol: e.symbol.clone(),
                        r#type: models::market_trade_good::Type::Import,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
            ]
        })
        .flatten()
        .flatten()
        .collect();
    insert_market_trade(&database_pool, market_trades).await;

    info!("Market scrapping workers done");
}
