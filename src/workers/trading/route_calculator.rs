// Route calculation and validation
use std::{
    collections::HashMap,
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
    ) -> anyhow::Result<ConcreteTradeRoute> {
        debug!("Getting new best route");
        let trade_goods = sql::MarketTradeGood::get_last(&self.context.database_pool).await?;

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
            .ok_or_else(|| anyhow::anyhow!("No routes found"))
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
            &mut self.cache.write().unwrap(),
        );

        let route_to = ship.find_route_cached(
            &waypoints,
            ship.nav.waypoint_symbol.clone(),
            trade_route.purchase_wp_symbol.clone(),
            &ship::nav_models::NavMode::BurnAndCruiseAndDrift,
            true,
            &mut self.cache.write().unwrap(),
        );

        let (_, _, total_fuel_cost, total_travel_time) =
            ship::stats::calc_route_stats(&waypoints, &route.unwrap(), ship.engine_speed);

        let (_, _, total_fuel_cost_to, total_travel_time_to) =
            ship::stats::calc_route_stats(&waypoints, &route_to.unwrap(), ship.engine_speed);

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
        total_travel_time: chrono::TimeDelta,
    ) -> TripStats {
        let trip_fuel_cost = (total_fuel_cost * 2) / 100 * CONFIG.trading.fuel_cost;

        let trip_volume = ship
            .cargo
            .capacity
            .min((trade_route.min_trade_volume as f32 * CONFIG.trading.purchase_multiplier) as i32);

        let trip_total_cost = trade_route.purchase_price * trip_volume + trip_fuel_cost;
        let trip_total_profit = trade_route.sell_price * trip_volume - trip_total_cost;

        let trip_per_hour = (total_travel_time.num_milliseconds() * 2) as f64
            / (chrono::TimeDelta::hours(1).num_milliseconds()) as f64;

        let profit_per_hour = trip_total_profit as f64 / trip_per_hour;

        TripStats {
            trip_time: total_travel_time * 2,
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
        total_travel_time: chrono::TimeDelta,
        total_fuel_cost_to: i32,
        total_travel_time_to: chrono::TimeDelta,
    ) -> TripStats {
        let trip_fuel_cost =
            (total_fuel_cost * total_fuel_cost_to) / 100 * CONFIG.trading.fuel_cost;

        let trip_volume = ship
            .cargo
            .capacity
            .min((trade_route.min_trade_volume as f32 * CONFIG.trading.purchase_multiplier) as i32);

        let trip_total_cost = trade_route.purchase_price * trip_volume + trip_fuel_cost;
        let trip_total_profit = trade_route.sell_price * trip_volume - trip_total_cost;

        let trip_per_hour = (total_travel_time.num_milliseconds()
            + total_travel_time_to.num_milliseconds()) as f64
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
}
