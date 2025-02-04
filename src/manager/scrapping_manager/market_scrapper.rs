use std::time::Duration;

use log::{debug, info};
use space_traders_client::models;
use tokio::time::sleep;

use crate::{config::CONFIG, types::WaypointCan, workers::market_scrapers::update_markets};

pub struct MarketScrapper {
    cancel_token: tokio_util::sync::CancellationToken,
    context: crate::workers::types::ConductorContext,
}

impl MarketScrapper {
    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: crate::workers::types::ConductorContext,
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
