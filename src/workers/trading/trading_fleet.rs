use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use futures::FutureExt;
use log::{debug, error, info};
use tokio_util::sync::CancellationToken;

use crate::{
    config::CONFIG,
    ship::{self, nav_models::Cache},
    sql,
    types::ConductorContext,
    workers::types::Conductor,
};

use super::{
    route_calculator::RouteCalculator, routes_track_keeper::RoutesTrackKeeper,
    trade_processor::TradeProcessor,
};

// Main TradingFleet implementation
#[derive(Debug, Clone)]
pub struct TradingFleet {
    context: ConductorContext,
    route_calculator: RouteCalculator,
    trade_executor: TradeProcessor,
    running_routes: Arc<RoutesTrackKeeper>,
    please_stop: CancellationToken,
}

impl TradingFleet {
    pub fn new_box(context: ConductorContext) -> Box<Self> {
        let cache = Arc::new(RwLock::new(Cache::default()));
        let running_routes = Arc::new(RoutesTrackKeeper::default());

        Box::new(TradingFleet {
            route_calculator: RouteCalculator::new(context.clone(), cache.clone()),
            trade_executor: TradeProcessor::new(context.clone(), running_routes.clone()),
            context,
            running_routes,
            please_stop: CancellationToken::new(),
        })
    }

    fn with_cancel(&self, _please_stop: CancellationToken) -> TradingFleet {
        TradingFleet {
            please_stop: _please_stop,
            ..self.clone()
        }
    }

    async fn run_trade_worker(&self) -> anyhow::Result<()> {
        info!("Starting trading workers");

        if !CONFIG.trading.active {
            info!("trading workers not active, exiting");

            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(CONFIG.trading.start_sleep_duration)).await;

        let ships = self.get_trading_ships().await;
        let mut handles = Vec::new();

        for ship in ships {
            let child_stopper = self.please_stop.child_token();
            let fleet = self.with_cancel(child_stopper.clone());
            let handle = tokio::spawn(async move { fleet.run_trade_ship_worker(ship).await }).then(
                |result| async move {
                    if let Err(e) = &result {
                        error!("Lel Error: {}", e);
                    }
                    if let Ok(result) = &result {
                        if let Err(e) = result {
                            error!(
                                "We got Error: {} {:?} {:?} {:?}",
                                e,
                                e.backtrace(),
                                e.source(),
                                e.root_cause()
                            );
                        }
                    }
                    result
                },
            );
            handles.push((handle, child_stopper));
        }

        info!("Waiting for trading workers to finish");

        for handle in handles {
            if let Err(e) = handle.0.await.unwrap() {
                info!("Error: {}", e);
                println!(
                    "Trade Error: {} {:?} {:?} {:?}",
                    e,
                    e.backtrace(),
                    e.source(),
                    e.root_cause()
                );
            }
        }

        info!("Trading workers done");
        Ok(())
    }

    async fn get_trading_ships(&self) -> Vec<String> {
        let ships = sql::ShipInfo::get_by_role(
            &self.context.database_pool,
            &crate::sql::ShipInfoRole::Trader,
        )
        .await
        .unwrap();
        ships.iter().map(|s| s.symbol.clone()).collect()
    }

    async fn run_trade_ship_worker(&self, ship_symbol: String) -> anyhow::Result<()> {
        let mut guard = self.context.ship_manager.get_mut(&ship_symbol).await;
        let ship = guard.value_mut().unwrap();

        debug!("Starting trade for {}", ship_symbol);
        tokio::time::sleep(std::time::Duration::from_millis(
            1000 + rand::random::<u64>() % 1000,
        ))
        .await;
        self.finish_trade_trade(ship).await?;
        let delay = 1000 + rand::random::<u64>() % 1000;
        debug!("Waiting for trade cycle for {} {}", ship_symbol, delay);

        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;

        for i in 0..CONFIG.trading.trade_cycle {
            if self.please_stop.is_cancelled() {
                info!("Trade cycle cancelled for {} ", ship_symbol);
                break;
            }
            let route: sql::TradeRoute = self
                .route_calculator
                .get_best_route(&ship, &self.running_routes)
                .await?;
            self.trade_executor
                .process_trade_route(ship, route.into(), i as i32)
                .await?;
        }

        Ok(())
    }

    async fn finish_trade_trade(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        let unfinished_trades =
            sql::TradeRoute::get_unfinished(&self.context.database_pool).await?;

        for trade_route in unfinished_trades {
            if (trade_route.ship_symbol == ship.symbol) && (trade_route.finished == false) {
                self.trade_executor
                    .process_trade_route(ship, trade_route, 0)
                    .await?;
            }
        }

        for (symbol, _) in ship.cargo.inventory.clone() {
            let trade_goods = sql::MarketTradeGood::get_last(&self.context.database_pool).await?;
            let mut routes = self
                .route_calculator
                .calc_possible_trade_routes(trade_goods)
                .into_iter()
                .filter(|route| &route.symbol == &symbol)
                .collect::<Vec<_>>();

            routes.sort_by_key(|route| route.sell_price);

            if let Some(trade_route) = routes.last() {
                let concrete = self
                    .route_calculator
                    .calc_concrete_trade_route(ship, trade_route.clone());
                if ship.cargo.has(&symbol) {
                    self.trade_executor
                        .process_trade_route(ship, concrete.into(), 0)
                        .await?;
                }
            }
        }

        Ok(())
    }
}

impl Conductor for TradingFleet {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_trade_worker().await })
    }

    fn get_name(&self) -> String {
        "TradingFleet".to_string()
    }

    fn get_cancel_token(&self) -> CancellationToken {
        self.please_stop.clone()
    }
}
