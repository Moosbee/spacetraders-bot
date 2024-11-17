use std::time::Duration;

use anyhow::Ok;
use log::{debug, info};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{
    sql::{self, DatabaseConnector},
    IsMarketplace,
};

const MAX_SCRAPS: u32 = 1;
const SCRAP_INTERVAL: u64 = 10;

pub struct MarketScraper {
    context: super::types::ConductorContext,
}

impl MarketScraper {
    #[allow(dead_code)]
    pub fn new_box(context: super::types::ConductorContext) -> Box<Self> {
        Box::new(MarketScraper { context })
    }

    async fn run_market_scraper(&self) -> anyhow::Result<()> {
        info!("Starting market scrapping workers");

        for i in 0..MAX_SCRAPS {
            if i != 0 {
                sleep(Duration::from_secs(SCRAP_INTERVAL)).await;
            }

            let markets = self.get_all_markets().await?;

            info!("Markets: {:?}", markets.len());
            update_markets(markets, self.context.database_pool.clone()).await;
        }

        info!("Market scrapping workers done");

        Ok(())
    }

    async fn get_all_markets(&self) -> anyhow::Result<Vec<models::Market>> {
        let future_markets = self
            .context
            .all_waypoints
            .iter()
            .map(|f| f.clone().into_iter())
            .flatten()
            .map(|w| w.1.clone())
            .filter(|w| w.is_marketplace())
            .map(|w| {
                let api = self.context.api.clone();
                let w = w.clone();
                tokio::spawn(async move {
                    debug!("Market: {:?}", w);
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

impl super::types::Conductor for MarketScraper {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_market_scraper().await })
    }

    fn get_name(&self) -> String {
        "MarketScraper".to_string()
    }
}

pub async fn update_markets(markets: Vec<models::Market>, database_pool: sqlx::PgPool) {
    let market_goods = markets
        .iter()
        .filter(|m| m.trade_goods.is_some())
        .map(|m| {
            m.trade_goods
                .clone()
                .unwrap()
                .iter()
                .map(|f| sql::MarketTradeGood::from(f.clone(), &m.symbol))
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();

    let market_transactions = markets
        .iter()
        .filter(|m| m.transactions.is_some())
        .map(|m| m.transactions.clone().unwrap())
        .flatten()
        .map(|mt| sql::MarketTransaction::try_from(mt).unwrap())
        .collect::<Vec<_>>();

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

pub async fn update_market(market: models::Market, database_pool: &sqlx::PgPool) {
    if let Some(trade_goods) = market.trade_goods {
        sql::MarketTradeGood::insert_bulk(
            &database_pool,
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
            &database_pool,
            &transactions
                .iter()
                .map(|f| sql::MarketTransaction::try_from(f.clone()).unwrap())
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();
    }

    let market_trades = vec![
        market
            .exchange
            .iter()
            .map(|e| sql::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol.clone(),
                r#type: models::market_trade_good::Type::Exchange,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
        market
            .exports
            .iter()
            .map(|e| sql::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol.clone(),
                r#type: models::market_trade_good::Type::Export,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
        market
            .imports
            .iter()
            .map(|e| sql::MarketTrade {
                waypoint_symbol: market.symbol.clone(),
                symbol: e.symbol.clone(),
                r#type: models::market_trade_good::Type::Import,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    ]
    .iter()
    .flatten()
    .map(|f| f.clone())
    .collect();
    sql::MarketTrade::insert_bulk(&database_pool, &market_trades)
        .await
        .unwrap();

    ()
}
