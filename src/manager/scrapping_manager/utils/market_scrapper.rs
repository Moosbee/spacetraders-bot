use std::time::Duration;

use database::DatabaseConnector;
use log::debug;
use space_traders_client::models;
use tokio::time::sleep;
use tracing::instrument;

#[instrument(skip(waypoints))]
pub async fn get_all_markets(
    api: &space_traders_client::Api,
    waypoints: &[(String, String, bool)],
) -> crate::error::Result<Vec<models::Market>> {
    let mut market_handles = tokio::task::JoinSet::new();

    for waypoint in waypoints {
        let api = api.clone();
        let waypoint = waypoint.clone();
        market_handles.spawn(async move {
            debug!("Market: {}", waypoint.1);

            loop {
                let market = api.get_market(&waypoint.0, &waypoint.1).await;

                match market {
                    Ok(market) => {
                        break *market.data;
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = format!("{} {:?}", e, e),
                            waypoint_symbol = waypoint.1,
                            "Market Error",
                        );
                        sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        });
    }

    let mut markets = Vec::new();

    while let Some(market_data) = market_handles.join_next().await {
        debug!(
            "Market: {} {}",
            market_data.is_ok(),
            market_data
                .as_ref()
                .map(|m| m.symbol.clone())
                .unwrap_or("Error".to_string())
        );

        match market_data {
            Ok(market) => {
                markets.push(market);
            }
            Err(e) => {
                tracing::warn!(error = format!("{} {:?}", e, e), "Market Join Error",);
            }
        }
    }

    Ok(markets)
}

pub async fn update_markets(
    markets: Vec<models::Market>,
    database_pool: database::DbPool,
) -> crate::error::Result<()> {
    let market_goods = markets
        .iter()
        .filter(|m| m.trade_goods.is_some())
        .flat_map(|m| {
            m.trade_goods
                .clone()
                .unwrap()
                .iter()
                .map(|f| database::MarketTradeGood::from(f.clone(), &m.symbol))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let market_transactions = markets
        .iter()
        .filter_map(|m| m.transactions.clone())
        .flatten()
        .map(|mt| database::MarketTransaction::try_from(mt).unwrap())
        .collect::<Vec<_>>();

    let market_trades: Vec<_> = markets
        .iter()
        .flat_map(|m| {
            vec![
                m.exchange
                    .iter()
                    .map(|e| database::MarketTrade {
                        waypoint_symbol: m.symbol.clone(),
                        symbol: e.symbol,
                        r#type: models::market_trade_good::Type::Exchange,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
                m.exports
                    .iter()
                    .map(|e| database::MarketTrade {
                        waypoint_symbol: m.symbol.clone(),
                        symbol: e.symbol,
                        r#type: models::market_trade_good::Type::Export,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
                m.imports
                    .iter()
                    .map(|e| database::MarketTrade {
                        waypoint_symbol: m.symbol.clone(),
                        symbol: e.symbol,
                        r#type: models::market_trade_good::Type::Import,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
            ]
        })
        .flatten()
        .collect();
    database::MarketTrade::insert_bulk(&database_pool, &market_trades).await?;
    database::MarketTradeGood::insert_bulk(&database_pool, &market_goods).await?;
    database::MarketTransaction::insert_bulk(&database_pool, &market_transactions).await?;

    Ok(())
}

pub async fn update_market(market: models::Market, database_pool: &database::DbPool) {
    if let Some(trade_goods) = market.trade_goods {
        database::MarketTradeGood::insert_bulk(
            database_pool,
            &trade_goods
                .iter()
                .map(|f| database::MarketTradeGood::from(f.clone(), &market.symbol))
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();
    }
    if let Some(transactions) = market.transactions {
        database::MarketTransaction::insert_bulk(
            database_pool,
            &transactions
                .iter()
                .map(|f| database::MarketTransaction::try_from(f.clone()).unwrap())
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();
    }

    let market_trades = [
        market
            .exchange
            .iter()
            .map(|e| database::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol,
                r#type: models::market_trade_good::Type::Exchange,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
        market
            .exports
            .iter()
            .map(|e| database::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol,
                r#type: models::market_trade_good::Type::Export,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
        market
            .imports
            .iter()
            .map(|e| database::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol,
                r#type: models::market_trade_good::Type::Import,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    ]
    .iter()
    .flatten()
    .cloned()
    .collect::<Vec<_>>();
    database::MarketTrade::insert_bulk(database_pool, &market_trades)
        .await
        .unwrap();
}
