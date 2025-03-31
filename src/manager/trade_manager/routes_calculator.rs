use std::collections::HashMap;

use log::debug;
use space_traders_client::models;

use crate::{
    config::CONFIG,
    error::Error,
    ship::{self},
    sql,
    types::ConductorContext,
};

use super::{
    route_calculator_concrete::ConcreteRouteCalculator,
    routes::{ExtrapolatedTradeRoute, PossibleTradeRoute, RouteData},
    routes_tracker::RoutesTracker,
};

#[derive(Debug)]
pub struct RouteCalculator {
    context: ConductorContext,
    concrete: ConcreteRouteCalculator,
}

impl RouteCalculator {
    pub fn new(context: ConductorContext) -> Self {
        Self {
            context: context.clone(),
            concrete: ConcreteRouteCalculator::new(context.clone()),
        }
    }

    /*

    New Thing, instead of calculatin seperate routes for simple and complex, we can combine them and have all routes and all the knowlege we have for them

    */

    pub async fn get_best_route(
        &mut self,
        ship: &ship::MyShip,
        running_routes: &RoutesTracker,
    ) -> Result<sql::TradeRoute, Error> {
        debug!("Getting new best route");
        let (trade_goods, market_trade) = self.fetch_market_data(&ship.nav.system_symbol).await?;

        let possible_trades = self.gen_all_possible_trades(&trade_goods, &market_trade);

        let waypoints =
            sql::Waypoint::get_by_system(&self.context.database_pool, &ship.nav.system_symbol)
                .await?;

        let routes = possible_trades
            .into_iter()
            .map(|route| self.extrapolate_trade_route(route))
            .filter(|route| self.is_valid_route(route))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|route| self.concrete.calc(ship, route, &waypoints))
            .filter(|route| route.data.profit > 0)
            .collect::<Vec<_>>();

        debug!("Routes: {}", routes.len());
        // debug!("Routes: {:#?}", routes);

        let route = routes
            .into_iter()
            .filter(|route| !running_routes.is_locked(&(*route).clone().into()))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .ok_or_else(|| Error::General("No routes main found".to_string()));

        route.map(|route| route.into())
    }

    async fn fetch_market_data(
        &self,
        system_symbol: &str,
    ) -> Result<(Vec<sql::MarketTradeGood>, Vec<sql::MarketTrade>), Error> {
        let trade_goods =
            sql::MarketTradeGood::get_last_by_system(&self.context.database_pool, system_symbol)
                .await?;
        let market_trade =
            sql::MarketTrade::get_last_by_system(&self.context.database_pool, system_symbol)
                .await?;
        Ok((trade_goods, market_trade))
    }

    pub fn gen_all_possible_trades<'a>(
        &self,
        trade_goods: &'a [sql::MarketTradeGood],
        market_trade: &'a [sql::MarketTrade],
    ) -> Vec<PossibleTradeRoute> {
        let trade_goods_map = trade_goods
            .iter()
            .map(|t| ((t.symbol, t.waypoint_symbol.clone()), t.clone()))
            .collect::<HashMap<(models::TradeSymbol, String), sql::MarketTradeGood>>();

        let possible_trades = market_trade
            .iter()
            .flat_map(|t| market_trade.iter().map(move |t2| (t, t2)))
            .filter(|t| t.0.symbol == t.1.symbol)
            .map(|(t1, t2)| {
                let trade_good_1 = trade_goods_map.get(&(t1.symbol, t1.waypoint_symbol.clone()));

                let trade_good_2 = trade_goods_map.get(&(t2.symbol, t2.waypoint_symbol.clone()));

                assert!(
                    t1.symbol == t2.symbol
                        && trade_good_1.map(|t| t.symbol).unwrap_or(t1.symbol)
                            == trade_good_2.map(|t| t.symbol).unwrap_or(t2.symbol)
                );

                PossibleTradeRoute {
                    symbol: t1.symbol,
                    purchase_good: trade_good_1.cloned(),
                    sell_good: trade_good_2.cloned(),
                    purchase: t1.clone(),
                    sell: t2.clone(),
                }
            })
            .collect::<Vec<_>>();

        possible_trades
    }

    fn extrapolate_trade_route(&self, route: PossibleTradeRoute) -> ExtrapolatedTradeRoute {
        let (min_trade_volume, max_trade_volume) = {
            let min_volume = route
                .purchase_good
                .as_ref()
                .map(|t| t.trade_volume)
                .unwrap_or(i32::MAX)
                .min(
                    route
                        .sell_good
                        .as_ref()
                        .map(|t| t.trade_volume)
                        .unwrap_or(i32::MAX),
                );

            let max_volume = route
                .purchase_good
                .as_ref()
                .map(|t| t.trade_volume)
                .unwrap_or(i32::MIN)
                .max(
                    route
                        .sell_good
                        .as_ref()
                        .map(|t| t.trade_volume)
                        .unwrap_or(i32::MIN),
                );

            (
                if min_volume == i32::MAX {
                    10
                } else {
                    min_volume
                },
                if max_volume == i32::MIN {
                    10
                } else {
                    max_volume
                },
            )
        };

        // Constants for price calculations

        let purchase_price: Option<i32> = route.purchase_good.as_ref().map(|t| t.purchase_price);
        let sell_price: Option<i32> = route.sell_good.as_ref().map(|t| t.sell_price);

        let (final_purchase_price, final_sell_price, final_profit) =
            match (purchase_price, sell_price) {
                (Some(p), Some(s)) => {
                    // Both prices are known
                    let profit = s - p;
                    (p, s, profit)
                }
                (Some(p), None) => {
                    // Only purchase price is known, apply markup
                    let markup = (p as f32 * CONFIG.trading.markup_percentage) as i32;
                    let estimated_sell = p + markup;
                    (p, estimated_sell, markup)
                }
                (None, Some(s)) => {
                    // Only sell price is known, apply margin
                    let margin = (s as f32 * CONFIG.trading.margin_percentage) as i32;
                    let estimated_purchase = s - margin;
                    (estimated_purchase, s, margin)
                }
                (None, None) => {
                    // No prices known, use default values
                    (
                        CONFIG.trading.default_purchase_price,
                        CONFIG.trading.default_sell_price,
                        CONFIG.trading.default_profit,
                    )
                }
            };

        let data = RouteData {
            min_trade_volume,
            max_trade_volume,
            purchase_price: final_purchase_price,
            sell_price: final_sell_price,
            profit: final_profit,
        };

        ExtrapolatedTradeRoute { route, data }
    }
    fn is_valid_route(&self, route: &ExtrapolatedTradeRoute) -> bool {
        !CONFIG.trading.blacklist.contains(&route.route.symbol)
            && route.data.profit > 0
            && route.route.purchase.waypoint_symbol != route.route.sell.waypoint_symbol
    }
}
