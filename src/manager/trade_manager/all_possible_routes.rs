use std::collections::HashMap;

use space_traders_client::models;

use crate::sql;

use super::routes::PossibleTradeRoute;

#[derive(Debug, Default)]
pub struct PossibleRoutes {}

impl PossibleRoutes {
    pub fn calc_possible_trade_routes(
        &self,
        trade_goods: Vec<sql::MarketTradeGood>,
    ) -> Vec<PossibleTradeRoute> {
        let trades = self.group_trade_goods_by_symbol(trade_goods);
        self.generate_trade_routes(&trades)
    }

    fn group_trade_goods_by_symbol(
        &self,
        trade_goods: Vec<sql::MarketTradeGood>,
    ) -> HashMap<models::TradeSymbol, (Vec<sql::MarketTradeGood>, Vec<sql::MarketTradeGood>)> {
        let mut trades = HashMap::new();

        for good in trade_goods {
            let entry = trades
                .entry(good.symbol)
                .or_insert_with(|| (Vec::new(), Vec::new()));

            entry.0.push(good.clone());
            entry.1.push(good);
        }

        trades
    }

    fn generate_trade_routes(
        &self,
        trades: &HashMap<
            models::TradeSymbol,
            (Vec<sql::MarketTradeGood>, Vec<sql::MarketTradeGood>),
        >,
    ) -> Vec<PossibleTradeRoute> {
        trades
            .iter()
            .flat_map(|(symbol, (exports, imports))| {
                self.generate_routes_for_symbol(*symbol, exports, imports)
            })
            .collect()
    }

    fn generate_routes_for_symbol<'a>(
        &'a self,
        symbol: models::TradeSymbol,
        exports: &'a [sql::MarketTradeGood],
        imports: &'a [sql::MarketTradeGood],
    ) -> impl Iterator<Item = PossibleTradeRoute> + '_ {
        exports.iter().flat_map(move |export| {
            imports.iter().map(move |import| PossibleTradeRoute {
                symbol,
                export: export.clone(),
                import: import.clone(),
                min_trade_volume: export.trade_volume.min(import.trade_volume),
                max_trade_volume: export.trade_volume.max(import.trade_volume),
                purchase_wp_symbol: export.waypoint_symbol.clone(),
                sell_wp_symbol: import.waypoint_symbol.clone(),
                purchase_price: export.purchase_price,
                sell_price: import.sell_price,
                profit: import.sell_price - export.purchase_price,
            })
        })
    }
}
