use std::time::Duration;

use log::{debug, info, warn};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{
    config::CONFIG,
    sql::{self, DatabaseConnector},
    types::{ConductorContext, WaypointCan},
};

use super::ScrappingManager;

pub struct MarketScrapper<'a> {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    scrapping_manager: &'a ScrappingManager,
}

impl<'a> MarketScrapper<'a> {
    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        scrapping_manager: &'a ScrappingManager,
    ) -> Self {
        Self {
            cancel_token,
            context,
            scrapping_manager,
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

            let markets_to_scrap = self.get_all_market_waypoints().await?;

            let markets = get_all_markets(&self.context.api, &markets_to_scrap).await?;

            info!("Markets: {:?}", markets.len());
            update_markets(markets, self.context.database_pool.clone()).await?;
        }

        info!("Market scrapping workers done");

        Ok(())
    }

    async fn get_all_market_waypoints(&self) -> crate::error::Result<Vec<(String, String)>> {
        let systems = self.scrapping_manager.get_system().await;

        let mut all_waypoints = vec![];
        debug!("Scrapping Systems: {}", systems.len());

        for system in systems {
            let waypoints =
                sql::Waypoint::get_by_system(&self.context.database_pool, &system).await?;

            let mut system_markets = waypoints
                .iter()
                .filter(|w| w.is_marketplace())
                .map(|w| (w.system_symbol.clone(), w.symbol.clone()))
                .collect::<Vec<_>>();

            all_waypoints.append(&mut system_markets);
        }

        Ok(all_waypoints)
    }
}

pub async fn get_all_markets(
    api: &crate::api::Api,
    waypoints: &[(String, String)],
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
                        warn!("Market: {} Error: {}", waypoint.1, e);
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
                warn!("Market: Error: {}", e);
            }
        }
    }

    Ok(markets)
}

pub async fn update_markets(
    markets: Vec<models::Market>,
    database_pool: sql::DbPool,
) -> crate::error::Result<()> {
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
    sql::MarketTrade::insert_bulk(&database_pool, &market_trades).await?;
    sql::MarketTradeGood::insert_bulk(&database_pool, &market_goods).await?;
    sql::MarketTransaction::insert_bulk(&database_pool, &market_transactions).await?;

    Ok(())
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
    .collect::<Vec<_>>();
    sql::MarketTrade::insert_bulk(database_pool, &market_trades)
        .await
        .unwrap();
}
