// Route calculation and validation
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use log::debug;
use space_traders_client::models;

use crate::{
    config::CONFIG,
    ship::{self, nav_models::Cache},
    sql,
    workers::{
        trading::{
            routes_track_keeper::RoutesTrackKeeper,
            t_types::{ConcreteTradeRoute, PossibleTradeRoute, TripStats},
        },
        types::ConductorContext,
    },
};

#[derive(Debug, Clone)]
pub struct RouteCalculator {
    context: ConductorContext,
    cache: Arc<RwLock<Cache>>,
}

impl RouteCalculator {
    pub fn new(context: ConductorContext, cache: Arc<RwLock<Cache>>) -> Self {
        Self { context, cache }
    }

    pub async fn get_best_route(
        &self,
        ship: &ship::MyShip,
        running_routes: &Arc<RoutesTrackKeeper>,
    ) -> anyhow::Result<sql::TradeRoute> {
        debug!("Getting new best route");
        let trade_goods = sql::MarketTradeGood::get_last(&self.context.database_pool).await?;
        let market_trade = sql::MarketTrade::get_last(&self.context.database_pool).await?;

        let waypoints_g = trade_goods
            .iter()
            .map(|t| t.waypoint_symbol.clone())
            .collect::<HashSet<String>>();
        let waypoints_m = market_trade
            .iter()
            .map(|t| t.waypoint_symbol.clone())
            .collect::<HashSet<String>>();

        let cache_ratio = waypoints_g.len() as f64 / waypoints_m.len() as f64;

        debug!(
            "Cache ratio: {} Trade: {} Market: {}",
            cache_ratio,
            waypoints_g.len(),
            waypoints_m.len()
        );

        let ratio: f64 = rand::random::<f64>();
        if ratio > cache_ratio {
            return self
                .get_routes_simple(market_trade, trade_goods, ship)
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No routes simple found"));
        }

        let routes = self
            .calc_possible_trade_routes(trade_goods)
            .into_iter()
            .filter(|route| !CONFIG.trading.blacklist.contains(&route.symbol) && route.profit > 0)
            .map(|route| self.calc_concrete_trade_route(ship, route))
            .filter(|route| route.profit > 0)
            .collect::<Vec<_>>();

        debug!("Routes: {}", routes.len());

        routes
            .iter()
            .filter(|route| !running_routes.is_locked(&(*route).clone().into()))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .cloned()
            .map(|route| route.into())
            .ok_or_else(|| anyhow::anyhow!("No routes main found"))
    }

    pub fn calc_possible_trade_routes(
        &self,
        trade_goods: Vec<sql::MarketTradeGood>,
    ) -> Vec<PossibleTradeRoute> {
        let mut trades: HashMap<
            models::TradeSymbol,
            (Vec<sql::MarketTradeGood>, Vec<sql::MarketTradeGood>),
        > = HashMap::new();

        // Group trade goods by symbol
        for good in trade_goods {
            let entry = trades
                .entry(good.symbol)
                .or_insert_with(|| (Vec::new(), Vec::new()));

            entry.0.push(good.clone());
            entry.1.push(good);
        }

        // Generate all possible routes
        trades
            .iter()
            .flat_map(|(symbol, (exports, imports))| {
                exports.iter().flat_map(|export| {
                    imports.iter().map(|import| PossibleTradeRoute {
                        symbol: *symbol,
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
            })
            .collect()
    }

    pub fn calc_concrete_trade_route(
        &self,
        ship: &ship::MyShip,
        trade_route: PossibleTradeRoute,
    ) -> ConcreteTradeRoute {
        let waypoints = self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .unwrap()
            .clone();

        let route = ship.find_route_cached(
            &waypoints,
            trade_route.sell_wp_symbol.clone(),
            trade_route.purchase_wp_symbol.clone(),
            &ship::nav_models::NavMode::BurnAndCruiseAndDrift,
            true,
            ship.fuel.capacity,
            &mut self.cache.write().unwrap(),
        );

        let route_to = ship.find_route_cached(
            &waypoints,
            ship.nav.waypoint_symbol.clone(),
            trade_route.purchase_wp_symbol.clone(),
            &ship::nav_models::NavMode::BurnAndCruiseAndDrift,
            true,
            ship.fuel.capacity,
            &mut self.cache.write().unwrap(),
        );

        let (_, _, total_fuel_cost, total_travel_time) = ship::stats::calc_route_stats(
            &waypoints,
            &route.unwrap(),
            ship.engine_speed,
            ship.conditions.engine.condition,
            ship.conditions.frame.condition,
            ship.conditions.reactor.condition,
        );

        let (_, _, total_fuel_cost_to, total_travel_time_to) = ship::stats::calc_route_stats(
            &waypoints,
            &route_to.unwrap(),
            ship.engine_speed,
            ship.conditions.engine.condition,
            ship.conditions.frame.condition,
            ship.conditions.reactor.condition,
        );

        let trip_stats = if true {
            self.calculate_reoccurring_trip_stats(
                ship,
                &trade_route,
                total_fuel_cost,
                total_travel_time,
            )
        } else {
            self.calculate_trip_stats(
                ship,
                &trade_route,
                total_fuel_cost,
                total_travel_time,
                total_fuel_cost_to,
                total_travel_time_to,
            )
        };

        ConcreteTradeRoute {
            symbol: trade_route.symbol,
            export: trade_route.export,
            import: trade_route.import,
            min_trade_volume: trade_route.min_trade_volume,
            max_trade_volume: trade_route.max_trade_volume,
            purchase_wp_symbol: trade_route.purchase_wp_symbol,
            sell_wp_symbol: trade_route.sell_wp_symbol,
            purchase_price: trade_route.purchase_price,
            sell_price: trade_route.sell_price,
            profit: trade_route.profit,
            ship_symbol: ship.symbol.clone(),
            trip_time: trip_stats.trip_time,
            trip_fuel_cost: trip_stats.trip_fuel_cost,
            trip_fuel_units: trip_stats.trip_fuel_units,
            trip_units: trip_stats.trip_units,
            trip_total_cost: trip_stats.trip_total_cost,
            trip_total_profit: trip_stats.trip_total_profit,
            trips_per_hour: trip_stats.trips_per_hour,
            profit_per_hour: trip_stats.profit_per_hour,
        }
    }

    fn calculate_reoccurring_trip_stats(
        &self,
        ship: &ship::MyShip,
        trade_route: &PossibleTradeRoute,
        total_fuel_cost: i32,
        total_travel_time: f64,
    ) -> TripStats {
        let trip_fuel_cost = (total_fuel_cost * 2) / 100 * CONFIG.trading.fuel_cost;

        let trip_volume = ship
            .cargo
            .capacity
            .min((trade_route.min_trade_volume as f32 * CONFIG.trading.purchase_multiplier) as i32);

        let trip_total_cost = trade_route.purchase_price * trip_volume + trip_fuel_cost;
        let trip_total_profit = trade_route.sell_price * trip_volume - trip_total_cost;

        let trip_per_hour = (total_travel_time * 1000.0 * 2.0)
            / (chrono::TimeDelta::hours(1).num_milliseconds()) as f64;

        let profit_per_hour = trip_total_profit as f64 / trip_per_hour;

        TripStats {
            trip_time: total_travel_time * 2.0,
            trip_fuel_cost,
            trip_fuel_units: total_fuel_cost * 2,
            trip_units: trip_volume,
            trip_total_cost,
            trip_total_profit,
            trips_per_hour: trip_per_hour as f32,
            profit_per_hour: profit_per_hour as i32,
        }
    }

    fn calculate_trip_stats(
        &self,
        ship: &ship::MyShip,
        trade_route: &PossibleTradeRoute,
        total_fuel_cost: i32,
        total_travel_time: f64,
        total_fuel_cost_to: i32,
        total_travel_time_to: f64,
    ) -> TripStats {
        let trip_fuel_cost =
            (total_fuel_cost * total_fuel_cost_to) / 100 * CONFIG.trading.fuel_cost;

        let trip_volume = ship
            .cargo
            .capacity
            .min((trade_route.min_trade_volume as f32 * CONFIG.trading.purchase_multiplier) as i32);

        let trip_total_cost = trade_route.purchase_price * trip_volume + trip_fuel_cost;
        let trip_total_profit = trade_route.sell_price * trip_volume - trip_total_cost;

        let trip_per_hour = (total_travel_time * 1000.0 + total_travel_time_to * 1000.0)
            / (chrono::TimeDelta::hours(1).num_milliseconds()) as f64;

        let profit_per_hour = trip_total_profit as f64 / trip_per_hour;

        TripStats {
            trip_time: total_travel_time + total_travel_time_to,
            trip_fuel_cost,
            trip_fuel_units: total_fuel_cost + total_fuel_cost_to,
            trip_units: trip_volume,
            trip_total_cost,
            trip_total_profit,
            trips_per_hour: trip_per_hour as f32,
            profit_per_hour: profit_per_hour as i32,
        }
    }

    fn get_routes_simple(
        &self,
        trade_goods: Vec<sql::MarketTrade>,
        detailed_trade_goods: Vec<sql::MarketTradeGood>,
        ship: &ship::MyShip,
    ) -> Vec<sql::TradeRoute> {
        let mut all_trades = trade_goods
            .iter()
            .map(|trade| {
                trade_goods
                    .iter()
                    .filter(|t| t.symbol == trade.symbol)
                    .map(|t| (trade.clone(), t.clone()))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .filter(|t| t.0.waypoint_symbol != t.1.waypoint_symbol)
            .filter(|t| {
                t.0.r#type == models::market_trade_good::Type::Export
                    && t.1.r#type == models::market_trade_good::Type::Import
            })
            .collect::<Vec<_>>();

        all_trades.sort_by(|a, b| {
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
        all_trades
            .iter()
            .map(|t| sql::TradeRoute {
                symbol: t.0.symbol.clone(),
                ship_symbol: ship.symbol.clone(),
                purchase_waypoint: t.0.waypoint_symbol.clone(),
                sell_waypoint: t.1.waypoint_symbol.clone(),
                finished: false,
                trade_volume: 20,
                ..Default::default()
            })
            .collect()
    }
}
