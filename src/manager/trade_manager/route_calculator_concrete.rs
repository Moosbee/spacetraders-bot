use crate::{
    config::CONFIG,
    ship::{self, nav_models::Cache},
};

use super::routes::{ConcreteTradeRoute, PossibleTradeRoute, TripStats};

#[derive(Debug)]
pub struct ConcreteRouteCalculator {
    context: crate::workers::types::ConductorContext,
    cache: Cache,
}

impl ConcreteRouteCalculator {
    pub fn new(context: crate::workers::types::ConductorContext, cache: Cache) -> Self {
        Self { context, cache }
    }
    pub fn calc(
        &mut self,
        ship: &ship::MyShip,
        trade_route: PossibleTradeRoute,
    ) -> ConcreteTradeRoute {
        let (route_stats, route_to_stats) = self.calculate_route_statistics(ship, &trade_route);

        let trip_stats = if true {
            self.calculate_reoccurring_trip_stats(
                ship,
                &trade_route,
                route_stats.fuel_cost,
                route_stats.travel_time,
            )
        } else {
            self.calculate_trip_stats(
                ship,
                &trade_route,
                route_stats.fuel_cost,
                route_stats.travel_time,
                route_to_stats.fuel_cost,
                route_to_stats.travel_time,
            )
        };

        self.create_concrete_route(ship, trade_route, trip_stats)
    }

    fn calculate_route_statistics(
        &mut self,
        ship: &ship::MyShip,
        trade_route: &PossibleTradeRoute,
    ) -> (RouteStats, RouteStats) {
        let waypoints = self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .unwrap()
            .clone();

        let route = self.find_route(
            ship,
            &waypoints,
            &trade_route.sell_wp_symbol,
            &trade_route.purchase_wp_symbol,
        );

        let route_to = self.find_route(
            ship,
            &waypoints,
            &ship.nav.waypoint_symbol,
            &trade_route.purchase_wp_symbol,
        );

        (
            self.calculate_single_route_stats(ship, &waypoints, &route.unwrap()),
            self.calculate_single_route_stats(ship, &waypoints, &route_to.unwrap()),
        )
    }

    fn create_concrete_route(
        &self,
        ship: &ship::MyShip,
        trade_route: PossibleTradeRoute,
        trip_stats: TripStats,
    ) -> ConcreteTradeRoute {
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

    fn find_route(
        &mut self,
        ship: &ship::MyShip,
        waypoints: &std::collections::HashMap<String, space_traders_client::models::Waypoint>,
        sell_wp_symbol: &str,
        purchase_wp_symbol: &str,
    ) -> Result<Vec<ship::nav_models::RouteConnection>, crate::error::Error> {
        let route = ship.find_route_cached(
            waypoints,
            sell_wp_symbol.to_string(),
            purchase_wp_symbol.to_string(),
            &ship::nav_models::NavMode::BurnAndCruiseAndDrift,
            true,
            ship.fuel.capacity,
            &mut self.cache,
        );
        route
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

    fn calculate_single_route_stats(
        &self,
        ship: &ship::MyShip,
        waypoints: &std::collections::HashMap<String, space_traders_client::models::Waypoint>,
        route: &[ship::nav_models::RouteConnection],
    ) -> RouteStats {
        let (_, _total_distance, total_fuel_cost, total_travel_time) =
            ship::stats::calc_route_stats(
                &waypoints,
                route,
                ship.engine_speed,
                ship.conditions.engine.condition,
                ship.conditions.frame.condition,
                ship.conditions.reactor.condition,
            );

        RouteStats {
            fuel_cost: total_fuel_cost as i32,
            travel_time: total_travel_time,
        }
    }
}

#[derive(Debug)]
struct RouteStats {
    fuel_cost: i32,
    travel_time: f64,
}
