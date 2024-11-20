use std::sync::{Arc, RwLock};

use log::{debug, info};

use crate::{
    ship::{self, nav_models::Cache},
    sql,
    workers::types::{Conductor, ConductorContext},
};

use super::{
    route_calculator::RouteCalculator, routes_track_keeper::RoutesTrackKeeper, t_types::constants,
    trade_processor::TradeProcessor,
};

// Main TradingFleet implementation
#[derive(Debug, Clone)]
pub struct TradingFleet {
    context: ConductorContext,
    route_calculator: RouteCalculator,
    trade_executor: TradeProcessor,
    running_routes: Arc<RoutesTrackKeeper>,
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
        })
    }

    async fn run_trade_worker(&self) -> anyhow::Result<()> {
        info!("Starting trading workers");

        let ships = self.get_trading_ships();
        let mut handles = Vec::new();

        for ship in ships {
            let fleet = self.clone();
            let handle = tokio::spawn(async move { fleet.run_trade_ship_worker(ship).await });
            handles.push(handle);
        }

        info!("Waiting for trading workers to finish");

        for handle in handles {
            if let Err(e) = handle.await.unwrap() {
                info!("Error: {}", e);
            }
        }

        info!("Trading workers done");
        Ok(())
    }

    fn get_trading_ships(&self) -> Vec<String> {
        self.context
            .ship_roles
            .iter()
            .filter(|(_, role)| **role == ship::models::Role::Trader)
            .map(|(symbol, _)| symbol.clone())
            .collect()
    }

    async fn run_trade_ship_worker(&self, ship_symbol: String) -> anyhow::Result<()> {
        let mut ship = self.context.my_ships.get_mut(&ship_symbol).unwrap();

        if ship.cargo.units > 0 {
            debug!("Starting cargo trade for {}", ship_symbol);
            self.finish_cargo_trade(&mut ship).await?;
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(
                1000 + rand::random::<u64>() % 1000,
            ))
            .await;
        }

        for _ in 0..constants::TRADE_CYCLE {
            let route = self
                .route_calculator
                .get_best_route(&ship, &self.running_routes)
                .await?;
            self.trade_executor
                .process_trade_route(&mut ship, route)
                .await?;
        }

        Ok(())
    }

    async fn finish_cargo_trade(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
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
                        .process_trade_route(ship, concrete)
                        .await?;
                }
            }
        }

        Ok(())
    }
}

impl Conductor for TradingFleet {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_trade_worker().await })
    }

    fn get_name(&self) -> String {
        "TradingFleet".to_string()
    }
}
