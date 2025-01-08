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

#[cfg(test)]
mod tests {
    use super::*;
    use space_traders_client::models::TradeSymbol;

    fn create_trade_good(
        symbol: TradeSymbol,
        waypoint: &str,
        purchase_price: i32,
        sell_price: i32,
        trade_volume: i32,
        r#type: models::market_trade_good::Type,
    ) -> sql::MarketTradeGood {
        sql::MarketTradeGood {
            symbol,
            waypoint_symbol: waypoint.to_string(),
            trade_volume,
            supply: models::SupplyLevel::Abundant,
            purchase_price,
            sell_price,
            r#type,
            activity: Some(models::ActivityLevel::Growing),
            created: sqlx::types::time::PrimitiveDateTime::MIN,
            created_at: sqlx::types::time::PrimitiveDateTime::MIN,
        }
    }

    #[test]
    fn test_empty_trade_goods() {
        let routes = PossibleRoutes::default();
        let result = routes.calc_possible_trade_routes(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_trade_good() {
        let routes = PossibleRoutes::default();
        let trade_good = create_trade_good(
            TradeSymbol::AdvancedCircuitry,
            "WP-1",
            100,
            150,
            10,
            models::market_trade_good::Type::Exchange,
        );
        let result = routes.calc_possible_trade_routes(vec![trade_good]);
        assert!(result.is_empty()); // Single trade good can't form a route
    }

    #[test]
    fn test_profitable_route() {
        let routes = PossibleRoutes::default();
        let export = create_trade_good(
            TradeSymbol::AdvancedCircuitry,
            "WP-1",
            100,
            150,
            10,
            models::market_trade_good::Type::Export,
        );
        let import = create_trade_good(
            TradeSymbol::AdvancedCircuitry,
            "WP-2",
            200,
            300,
            15,
            models::market_trade_good::Type::Import,
        );

        let mut result = routes.calc_possible_trade_routes(vec![export.clone(), import.clone()]);

        result.sort();
        result.reverse();

        println!("result: {:?}", result);

        assert_eq!(result.len(), 4);
        let route = &result[0];
        assert_eq!(route.symbol, TradeSymbol::AdvancedCircuitry);
        assert_eq!(route.purchase_wp_symbol, "WP-1");
        assert_eq!(route.sell_wp_symbol, "WP-2");
        assert_eq!(route.min_trade_volume, 10);
        assert_eq!(route.max_trade_volume, 15);
        assert_eq!(route.purchase_price, 100);
        assert_eq!(route.sell_price, 300);
        assert_eq!(route.profit, 200);
    }

    #[test]
    fn test_multiple_routes_same_symbol() {
        let routes = PossibleRoutes::default();
        let goods = vec![
            create_trade_good(
                TradeSymbol::AdvancedCircuitry,
                "WP-1",
                100,
                150,
                10,
                models::market_trade_good::Type::Import,
            ),
            create_trade_good(
                TradeSymbol::AdvancedCircuitry,
                "WP-2",
                200,
                300,
                15,
                models::market_trade_good::Type::Exchange,
            ),
            create_trade_good(
                TradeSymbol::AdvancedCircuitry,
                "WP-3",
                250,
                350,
                20,
                models::market_trade_good::Type::Export,
            ),
        ];

        let mut result = routes.calc_possible_trade_routes(goods);

        result.sort();

        println!("result: {:?}", result);

        // Should generate routes between all possible pairs
        assert_eq!(result.len(), 9);
        let route = &result[0];

        assert_eq!(route.symbol, TradeSymbol::AdvancedCircuitry);
        assert_eq!(route.purchase_wp_symbol, "WP-3");
        assert_eq!(route.sell_wp_symbol, "WP-1");
        assert_eq!(route.min_trade_volume, 10);
        assert_eq!(route.max_trade_volume, 20);
        assert_eq!(route.purchase_price, 250);
        assert_eq!(route.sell_price, 150);
        assert_eq!(route.profit, -100);
    }

    #[test]
    fn test_multiple_symbols() {
        let routes = PossibleRoutes::default();
        let goods = vec![
            create_trade_good(
                TradeSymbol::AdvancedCircuitry,
                "WP-1",
                100,
                150,
                10,
                models::market_trade_good::Type::Export,
            ),
            create_trade_good(
                TradeSymbol::AdvancedCircuitry,
                "WP-2",
                200,
                300,
                15,
                models::market_trade_good::Type::Import,
            ),
            create_trade_good(
                TradeSymbol::Iron,
                "WP-3",
                50,
                75,
                20,
                models::market_trade_good::Type::Export,
            ),
            create_trade_good(
                TradeSymbol::Iron,
                "WP-4",
                100,
                150,
                25,
                models::market_trade_good::Type::Exchange,
            ),
        ];

        let result = routes.calc_possible_trade_routes(goods);

        // Should generate routes for each symbol separately
        assert_eq!(result.len(), 8); // 4 routes for each symbol
    }

    #[test]
    fn test_trade_volume_calculation() {
        let routes = PossibleRoutes::default();
        let export = create_trade_good(
            TradeSymbol::AdvancedCircuitry,
            "WP-1",
            100,
            150,
            10,
            models::market_trade_good::Type::Exchange,
        );
        let import = create_trade_good(
            TradeSymbol::AdvancedCircuitry,
            "WP-2",
            200,
            300,
            20,
            models::market_trade_good::Type::Exchange,
        );

        let mut result = routes.calc_possible_trade_routes(vec![export, import]);

        result.sort();

        println!("result: {:?}", result);

        assert_eq!(result[3].min_trade_volume, 10);
        assert_eq!(result[3].max_trade_volume, 20);
    }
}
