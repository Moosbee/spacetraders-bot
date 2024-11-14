use anyhow::Ok;
use log::{debug, info};
use space_traders_client::models;

use crate::{
    api,
    sql::{self, insert_market_trade, insert_market_trade_good, insert_market_transactions},
    IsMarketplace,
};

pub struct MarketScraper {
    context: super::types::ConductorContext,
}

impl MarketScraper {
    pub fn new_box(context: super::types::ConductorContext) -> Box<Self> {
        Box::new(MarketScraper { context })
    }
}

impl super::types::Conductor for MarketScraper {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move {
            info!("Starting market scrapping workers");

            // sleep(Duration::from_secs(10)).await;

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

            info!("Markets: {:?}", markets.len());
            update_markets(markets, self.context.database_pool.clone()).await;

            info!("Market scrapping workers done");

            Ok(())
        })
    }

    fn get_name(&self) -> String {
        "MarketScraper".to_string()
    }
}

pub async fn update_markets(markets: Vec<models::Market>, database_pool: sqlx::PgPool) {
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
}

pub async fn update_market(market: models::Market, database_pool: &sqlx::PgPool) {
    if let Some(trade_goods) = market.trade_goods {
        insert_market_trade_good(
            &database_pool,
            trade_goods
                .iter()
                .map(|f| (market.symbol.clone(), f.clone()))
                .collect::<Vec<_>>(),
        )
        .await;
    }
    if let Some(transactions) = market.transactions {
        insert_market_transactions(
            &database_pool,
            transactions
                .iter()
                .map(|f| sql::MarketTransaction::try_from(f.clone()).unwrap())
                .collect::<Vec<_>>(),
        )
        .await;
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
    insert_market_trade(&database_pool, market_trades).await;

    ()
}
