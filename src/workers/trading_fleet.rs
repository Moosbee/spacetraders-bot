use core::fmt;
use std::collections::HashMap;

use log::{debug, info};
use space_traders_client::models::{self, TradeSymbol};

use crate::sql::{self, get_last_market_trade_goods};

pub struct TradingFleet {
    context: super::types::ConductorContext,
}

impl TradingFleet {
    pub fn new_box(context: super::types::ConductorContext) -> Box<Self> {
        Box::new(TradingFleet { context })
    }
}

impl super::types::Conductor for TradingFleet {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move {
            info!("Starting trading workers");
            let trade_goods: Vec<sql::MarketTradeGood> =
                get_last_market_trade_goods(&self.context.database_pool).await;
            let mut routes = calc_possible_trade_routes(trade_goods);
            routes.sort();
            for route in routes {
                println!("Route: {}", route);
            }

            info!("Trading workers done");

            Ok(())
        })
    }

    fn get_name(&self) -> String {
        "TradingFleet".to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PossibleTradeRoute {
    symbol: TradeSymbol,
    export: sql::MarketTradeGood,
    import: sql::MarketTradeGood,
    min_trade_volume: i32,
    max_trade_volume: i32,
    purchase_wp_symbol: String,
    sell_wp_symbol: String,
    purchase_price: i32,
    sell_price: i32,
    profit: i32,
}

impl PartialOrd for PossibleTradeRoute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.profit.partial_cmp(&other.profit)
    }
}

impl Ord for PossibleTradeRoute {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.profit.cmp(&other.profit)
    }
}

impl fmt::Display for PossibleTradeRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} -> {} {}",
            self.symbol, self.purchase_wp_symbol, self.sell_wp_symbol, self.profit
        )
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
