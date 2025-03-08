use std::time::Duration;

use log::{debug, info};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{
    config::CONFIG,
    sql::{self, DatabaseConnector},
    types::{ConductorContext, WaypointCan},
};

pub struct MarketScrapper {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
}

impl MarketScrapper {
    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
    ) -> Self {
        Self {
            cancel_token,
            context,
        }
    }
    pub async fn run_scrapping_worker(&self) -> crate::error::Result<()> {
        info!("Starting market scrapping workers");

        if !CONFIG.market.active {
            info!("Market scrapping not active, exiting");

            return Ok(());
        }

        for i in 0..CONFIG.market.max_scraps {
            if i != 0 {
                let erg = tokio::select! {
                _ = self.cancel_token.cancelled() => {
                  info!("Market scrapping cancelled");
                  0},
                _ =  sleep(Duration::from_millis(CONFIG.market.scrap_interval)) => {1},
                };
                if erg == 0 {
                    break;
                }
            }

            let markets = self.get_all_markets().await?;

            info!("Markets: {:?}", markets.len());
            update_markets(markets, self.context.database_pool.clone()).await;
        }

        info!("Market scrapping workers done");

        Ok(())
    }

    async fn get_all_markets(&self) -> crate::error::Result<Vec<models::Market>> {
        let future_markets = self
            .context
            .all_waypoints
            .iter()
            .flat_map(|f| f.clone().into_iter())
            .map(|w| w.1.clone())
            .filter(|w| w.is_marketplace())
            .map(|w| {
                let api = self.context.api.clone();
                let w = w.clone();
                tokio::spawn(async move {
                    debug!("Market: {}", w.symbol);
                    api.get_market(&w.system_symbol, &w.symbol).await.unwrap()
                })
            })
            .collect::<Vec<_>>();

        let mut markets = Vec::new();

        for market in future_markets {
            let market_data = market.await.unwrap().data;
            debug!("Market: {:?}", market_data.symbol);
            markets.push(*market_data);
        }

        Ok(markets)
    }
}

pub async fn update_markets(markets: Vec<models::Market>, database_pool: sql::DbPool) {
    let market_goods = markets
        .iter()
        .filter(|m| m.trade_goods.is_some())
        .flat_map(|m| {
            m.trade_goods
                .clone()
                .unwrap()
                .iter()
                .map(|f| sql::MarketTradeGood::from(f.clone(), &m.symbol))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let market_transactions = markets
        .iter()
        .filter_map(|m| m.transactions.clone())
        .flatten()
        .map(|mt| sql::MarketTransaction::try_from(mt).unwrap())
        .collect::<Vec<_>>();

    let market_trades: Vec<_> = markets
        .iter()
        .flat_map(|m| {
            vec![
                m.exchange
                    .iter()
                    .map(|e| sql::MarketTrade {
                        waypoint_symbol: m.symbol.clone(),
                        symbol: e.symbol,
                        r#type: models::market_trade_good::Type::Exchange,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
                m.exports
                    .iter()
                    .map(|e| sql::MarketTrade {
                        waypoint_symbol: m.symbol.clone(),
                        symbol: e.symbol,
                        r#type: models::market_trade_good::Type::Export,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
                m.imports
                    .iter()
                    .map(|e| sql::MarketTrade {
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
    sql::MarketTrade::insert_bulk(&database_pool, &market_trades)
        .await
        .unwrap();
    sql::MarketTradeGood::insert_bulk(&database_pool, &market_goods)
        .await
        .unwrap();
    sql::MarketTransaction::insert_bulk(&database_pool, &market_transactions)
        .await
        .unwrap();
}

pub async fn update_market(market: models::Market, database_pool: &sql::DbPool) {
    if let Some(trade_goods) = market.trade_goods {
        sql::MarketTradeGood::insert_bulk(
            database_pool,
            &trade_goods
                .iter()
                .map(|f| sql::MarketTradeGood::from(f.clone(), &market.symbol))
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();
    }
    if let Some(transactions) = market.transactions {
        sql::MarketTransaction::insert_bulk(
            database_pool,
            &transactions
                .iter()
                .map(|f| sql::MarketTransaction::try_from(f.clone()).unwrap())
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();
    }

    let market_trades = [
        market
            .exchange
            .iter()
            .map(|e| sql::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol,
                r#type: models::market_trade_good::Type::Exchange,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
        market
            .exports
            .iter()
            .map(|e| sql::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol,
                r#type: models::market_trade_good::Type::Export,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
        market
            .imports
            .iter()
            .map(|e| sql::MarketTrade {
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
    .collect();
    sql::MarketTrade::insert_bulk(database_pool, &market_trades)
        .await
        .unwrap();
}
