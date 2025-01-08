use std::collections::HashSet;

use log::debug;

use crate::{
    config::CONFIG,
    error::Error,
    ship::{self, nav_models::Cache},
    sql,
    workers::types::ConductorContext,
};

use super::{
    all_possible_routes::PossibleRoutes, route_calculator_concrete::ConcreteRouteCalculator,
    route_calculator_simple::RouteCalculatorSimple, routes::PossibleTradeRoute,
    routes_tracker::RoutesTracker,
};

#[derive(Debug)]
pub struct RouteCalculator {
    context: ConductorContext,
    simple: RouteCalculatorSimple,
    possible_routes: PossibleRoutes,
    concrete: ConcreteRouteCalculator,
}

impl RouteCalculator {
    pub fn new(context: ConductorContext, cache: Cache) -> Self {
        Self {
            context: context.clone(),
            simple: RouteCalculatorSimple {},
            possible_routes: PossibleRoutes::default(),
            concrete: ConcreteRouteCalculator::new(context.clone(), cache),
        }
    }

    pub async fn get_best_route(
        &mut self,
        ship: &ship::MyShip,
        running_routes: &RoutesTracker,
    ) -> Result<sql::TradeRoute, Error> {
        debug!("Getting new best route");
        let (trade_goods, market_trade) = self.fetch_market_data().await?;

        if self.should_use_simple_routes(&trade_goods, &market_trade) {
            return self
                .simple
                .get_routes_simple(&market_trade, &trade_goods, &ship.symbol)
                .first()
                .cloned()
                .ok_or_else(|| Error::General(format!("No routes simple found")));
        }

        self.calculate_best_complex_route(trade_goods, ship, running_routes)
    }

    async fn fetch_market_data(
        &self,
    ) -> Result<(Vec<sql::MarketTradeGood>, Vec<sql::MarketTrade>), Error> {
        let trade_goods = sql::MarketTradeGood::get_last(&self.context.database_pool).await?;
        let market_trade = sql::MarketTrade::get_last(&self.context.database_pool).await?;
        Ok((trade_goods, market_trade))
    }

    fn should_use_simple_routes(
        &self,
        trade_goods: &[sql::MarketTradeGood],
        market_trade: &[sql::MarketTrade],
    ) -> bool {
        let waypoints_g: HashSet<_> = trade_goods.iter().map(|t| &t.waypoint_symbol).collect();
        let waypoints_m: HashSet<_> = market_trade.iter().map(|t| &t.waypoint_symbol).collect();

        let cache_ratio = waypoints_g.len() as f64 / waypoints_m.len() as f64;
        debug!(
            "Cache ratio: {} Trade: {} Market: {}",
            cache_ratio,
            waypoints_g.len(),
            waypoints_m.len()
        );

        rand::random::<f64>() > cache_ratio
    }

    fn calculate_best_complex_route(
        &mut self,
        trade_goods: Vec<sql::MarketTradeGood>,
        ship: &ship::MyShip,
        running_routes: &RoutesTracker,
    ) -> Result<sql::TradeRoute, Error> {
        let possible_routes = self.possible_routes.calc_possible_trade_routes(trade_goods);
        let routes = possible_routes
            .into_iter()
            .filter(|route| self.is_valid_route(route))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|route| self.concrete.calc(ship, route))
            .filter(|route| route.profit > 0)
            .collect::<Vec<_>>();

        debug!("Routes: {}", routes.len());

        let route = routes
            .iter()
            .filter(|route| !running_routes.is_locked(&(*route).clone().into()))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .cloned()
            .map(|route| route)
            .ok_or_else(|| Error::General(format!("No routes main found")));

        route.map(|route| route.into())
    }

    fn is_valid_route(&self, route: &PossibleTradeRoute) -> bool {
        !CONFIG.trading.blacklist.contains(&route.symbol) && route.profit > 0
    }
}
