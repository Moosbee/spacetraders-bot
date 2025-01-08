use space_traders_client::models;

use crate::sql;

#[derive(Debug)]
pub struct RouteCalculatorSimple;

impl RouteCalculatorSimple {
    pub fn get_routes_simple(
        &self,
        trade_goods: &[sql::MarketTrade],
        detailed_trade_goods: &[sql::MarketTradeGood],
        ship_symbol: &str,
    ) -> Vec<sql::TradeRoute> {
        let mut all_trades = self.generate_all_possible_trades(trade_goods);
        self.sort_trades_by_detailed_goods(&mut all_trades, detailed_trade_goods);

        all_trades
            .iter()
            .map(|t| self.create_trade_route(t, ship_symbol))
            .collect()
    }

    fn generate_all_possible_trades(
        &self,
        trade_goods: &[sql::MarketTrade],
    ) -> Vec<(sql::MarketTrade, sql::MarketTrade)> {
        trade_goods
            .iter()
            .flat_map(|trade| {
                trade_goods
                    .iter()
                    .filter(|t| t.symbol == trade.symbol)
                    .map(|t| (trade.clone(), t.clone()))
            })
            .filter(|t| {
                t.0.waypoint_symbol != t.1.waypoint_symbol
                    && t.0.r#type == models::market_trade_good::Type::Export
                    && t.1.r#type == models::market_trade_good::Type::Import
            })
            .collect()
    }

    fn sort_trades_by_detailed_goods(
        &self,
        trades: &mut Vec<(sql::MarketTrade, sql::MarketTrade)>,
        detailed_trade_goods: &[sql::MarketTradeGood],
    ) {
        trades.sort_by(|a, b| {
            if detailed_trade_goods
                .iter()
                .any(|d| d.waypoint_symbol == a.0.waypoint_symbol)
            {
                return std::cmp::Ordering::Greater;
            } else if detailed_trade_goods
                .iter()
                .any(|d| d.waypoint_symbol == b.0.waypoint_symbol)
            {
                return std::cmp::Ordering::Greater;
            }
            return std::cmp::Ordering::Less;
        });
    }

    fn create_trade_route(
        &self,
        trade: &(sql::MarketTrade, sql::MarketTrade),
        ship_symbol: &str,
    ) -> sql::TradeRoute {
        sql::TradeRoute {
            symbol: trade.0.symbol.clone(),
            ship_symbol: ship_symbol.to_string(),
            purchase_waypoint: trade.0.waypoint_symbol.clone(),
            sell_waypoint: trade.1.waypoint_symbol.clone(),
            finished: false,
            trade_volume: 20,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use space_traders_client::models::{self, market_trade_good};

    fn create_market_trade(
        symbol: models::TradeSymbol,
        waypoint: &str,
        trade_type: market_trade_good::Type,
    ) -> sql::MarketTrade {
        sql::MarketTrade {
            symbol,
            waypoint_symbol: waypoint.to_string(),
            r#type: trade_type,
            created_at: sqlx::types::time::PrimitiveDateTime::MIN,
        }
    }

    fn create_detailed_trade(
        symbol: models::TradeSymbol,
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
        let calculator = RouteCalculatorSimple;
        let result = calculator.get_routes_simple(&[], &[], &"SHIP-1");
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_market_no_routes() {
        let calculator = RouteCalculatorSimple;
        let trades = vec![create_market_trade(
            models::TradeSymbol::AdvancedCircuitry,
            "WP-1",
            market_trade_good::Type::Export,
        )];
        let result = calculator.get_routes_simple(&trades, &[], &"SHIP-1");
        assert!(result.is_empty());
    }

    #[test]
    fn test_valid_trade_route() {
        let calculator = RouteCalculatorSimple;
        let trades = vec![
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-1",
                market_trade_good::Type::Export,
            ),
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-2",
                market_trade_good::Type::Import,
            ),
        ];

        let ship = "SHIP-1";
        let result = calculator.get_routes_simple(&trades, &[], &ship);

        assert_eq!(result.len(), 1);
        let route = &result[0];
        assert_eq!(route.symbol, models::TradeSymbol::AdvancedCircuitry);
        assert_eq!(route.ship_symbol, "SHIP-1");
        assert_eq!(route.purchase_waypoint, "WP-1");
        assert_eq!(route.sell_waypoint, "WP-2");
        assert_eq!(route.trade_volume, 20);
        assert!(!route.finished);
    }

    #[test]
    fn test_multiple_valid_routes() {
        let calculator = RouteCalculatorSimple;
        let trades = vec![
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-1",
                market_trade_good::Type::Export,
            ),
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-2",
                market_trade_good::Type::Import,
            ),
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-3",
                market_trade_good::Type::Import,
            ),
        ];

        let result = calculator.get_routes_simple(&trades, &[], &"SHIP-1");
        assert_eq!(result.len(), 2); // Should create routes from WP-1 to both WP-2 and WP-3
    }

    #[test]
    fn test_sorting_with_detailed_goods() {
        let calculator = RouteCalculatorSimple;
        let trades = vec![
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-1",
                market_trade_good::Type::Export,
            ),
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-2",
                market_trade_good::Type::Export,
            ),
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-3",
                market_trade_good::Type::Import,
            ),
        ];

        let detailed_goods = vec![create_detailed_trade(
            models::TradeSymbol::AdvancedCircuitry,
            "WP-2",
            100,
            200,
            10,
            market_trade_good::Type::Import,
        )];

        let result = calculator.get_routes_simple(&trades, &detailed_goods, &"SHIP-1");
        assert!(!result.is_empty());

        println!("result: {:?}", result);
        // Routes from WP-1 should be prioritized due to having less detailed trade information, so it should be first
        assert_eq!(result[0].purchase_waypoint, "WP-1");
    }

    #[test]
    fn test_filter_invalid_routes() {
        let calculator = RouteCalculatorSimple;
        let trades = vec![
            // Same waypoint export/import should be filtered out
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-1",
                market_trade_good::Type::Export,
            ),
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-1",
                market_trade_good::Type::Import,
            ),
            // Import to Export should be filtered out
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-2",
                market_trade_good::Type::Import,
            ),
            create_market_trade(
                models::TradeSymbol::AdvancedCircuitry,
                "WP-3",
                market_trade_good::Type::Export,
            ),
        ];

        let result = calculator.get_routes_simple(&trades, &[], &"SHIP-1");

        assert!(result.len() == 3);
    }
}
