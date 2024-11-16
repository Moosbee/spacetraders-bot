use std::collections::HashMap;

use log::{debug, info};
use space_traders_client::models;

use crate::sql;

use super::types::PossibleTradeRoute;

const BLACKLIST: [models::TradeSymbol; 2] = [
    models::TradeSymbol::AdvancedCircuitry,
    models::TradeSymbol::FabMats,
];

pub struct TradingFleet {
    context: super::types::ConductorContext,
}

impl TradingFleet {
    #[allow(dead_code)]
    pub fn new_box(context: super::types::ConductorContext) -> Box<Self> {
        Box::new(TradingFleet { context })
    }

    async fn run_trade_worker(&self) -> anyhow::Result<()> {
        info!("Starting trading workers");
        let trade_goods: Vec<sql::MarketTradeGood> =
            sql::MarketTradeGood::get_last(&self.context.database_pool).await?;
        let mut routes = calc_possible_trade_routes(trade_goods)
            .into_iter()
            .filter(|route| !BLACKLIST.contains(&route.symbol))
            .collect::<Vec<_>>();
        routes.sort();
        for route in routes {
            info!("Route: {}", route);
        }

        info!("Trading workers done");

        Ok(())
    }

    async fn process_trade_route(&self, route: PossibleTradeRoute) {}
}

impl super::types::Conductor for TradingFleet {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_trade_worker().await })
    }

    fn get_name(&self) -> String {
        "TradingFleet".to_string()
    }
}

fn calc_possible_trade_routes(trade_goods: Vec<sql::MarketTradeGood>) -> Vec<PossibleTradeRoute> {
    let mut trades: HashMap<
        models::TradeSymbol,
        (Vec<sql::MarketTradeGood>, Vec<sql::MarketTradeGood>),
    > = HashMap::new();
    for t in trade_goods {
        debug!("Trade: {:?}", t);
        let v = trades.get(&t.symbol);

        let mut trade: (Vec<_>, Vec<_>) = v.clone().unwrap_or(&(Vec::new(), Vec::new())).clone();
        if t.r#type == models::market_trade_good::Type::Export
            || t.r#type == models::market_trade_good::Type::Exchange
        {
            trade.0.push(t.clone().into());
        }
        if t.r#type == models::market_trade_good::Type::Import
            || t.r#type == models::market_trade_good::Type::Exchange
        {
            trade.1.push(t.clone().into());
        }
        trades.insert(t.symbol, trade);
    }

    trades
        .iter()
        .map(|(trade_good, (exports, imports))| {
            let exports: Vec<_> = exports
                .iter()
                .map(|export| {
                    let imports: Vec<_> = imports
                        .iter()
                        .map(|import| {
                            // (trade_good.clone(), export.clone(), import.clone())

                            PossibleTradeRoute {
                                symbol: trade_good.clone(),
                                export: export.clone(),
                                import: import.clone(),
                                min_trade_volume: export.trade_volume.min(import.trade_volume),
                                max_trade_volume: export.trade_volume.max(import.trade_volume),
                                purchase_wp_symbol: export.waypoint_symbol.clone(),
                                sell_wp_symbol: import.waypoint_symbol.clone(),
                                purchase_price: export.purchase_price,
                                sell_price: import.sell_price,
                                profit: import.sell_price - export.purchase_price,
                            }
                        })
                        .collect();
                    imports
                })
                .collect();
            exports
        })
        .flatten()
        .flatten()
        .collect()
}
