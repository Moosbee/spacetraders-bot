use std::time::Duration;

use anyhow::Ok;
use log::{debug, info};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{
    config::CONFIG,
    sql::{self, DatabaseConnector},
    IsMarketplace,
};

#[derive(Debug, Clone)]
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

        if !CONFIG.market.active {
            info!("Market scrapping not active, exiting");

            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(CONFIG.market.start_sleep_duration)).await;

        let handle = if CONFIG.market.agents {
            let my_agent = self.clone();
            tokio::spawn(async move { my_agent.run_agent_scraper().await })
        } else {
            tokio::spawn(async move { Ok(()) })
        };

        for i in 0..CONFIG.market.max_scraps {
            if i != 0 {
                sleep(Duration::from_millis(CONFIG.market.scrap_interval)).await;
            }

            let markets = self.get_all_markets().await?;

            info!("Markets: {:?}", markets.len());
            update_markets(markets, self.context.database_pool.clone()).await;
        }

        handle.await??;

        info!("Market scrapping workers done");

        Ok(())
    }

    pub async fn run_agent_scraper(&self) -> anyhow::Result<()> {
        for _i in 0..CONFIG.market.max_agent_scraps {
            self.update_all_agents().await?;
            sleep(Duration::from_millis(CONFIG.market.agent_interval)).await;
        }

        Ok(())
    }

    async fn update_all_agents(&self) -> anyhow::Result<()> {
        let agents = self.context.api.get_all_agents(20).await?;
        let all_agents = agents.into_iter().map(sql::Agent::from).collect::<Vec<_>>();

        for agent in &all_agents {
            sql::Agent::insert(&self.context.database_pool, &agent).await?;
        }

        Ok(())
    }

    async fn get_all_markets(&self) -> anyhow::Result<Vec<models::Market>> {
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

pub async fn update_market(market: models::Market, database_pool: &sqlx::PgPool) {
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
