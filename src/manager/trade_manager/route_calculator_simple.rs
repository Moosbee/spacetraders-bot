use space_traders_client::models;

use crate::{ship, sql};

#[derive(Debug)]
pub struct RouteCalculatorSimple;

impl RouteCalculatorSimple {
    pub fn get_routes_simple(
        &self,
        trade_goods: &[sql::MarketTrade],
        detailed_trade_goods: &[sql::MarketTradeGood],
        ship: &ship::MyShip,
    ) -> Vec<sql::TradeRoute> {
        let mut all_trades = self.generate_all_possible_trades(trade_goods);
        self.sort_trades_by_detailed_goods(&mut all_trades, detailed_trade_goods);

        all_trades
            .iter()
            .map(|t| self.create_trade_route(t, ship))
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
        ship: &ship::MyShip,
    ) -> sql::TradeRoute {
        sql::TradeRoute {
            symbol: trade.0.symbol.clone(),
            ship_symbol: ship.symbol.clone(),
            purchase_waypoint: trade.0.waypoint_symbol.clone(),
            sell_waypoint: trade.1.waypoint_symbol.clone(),
            finished: false,
            trade_volume: 20,
            ..Default::default()
        }
    }
}
