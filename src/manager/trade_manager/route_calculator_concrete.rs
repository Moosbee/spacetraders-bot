use std::collections::HashMap;

use crate::{
    config::CONFIG,
    ship::{self},
    utils::ConductorContext,
};

use super::routes::{ConcreteTradeRoute, ExtrapolatedTradeRoute, TripStats};

#[derive(Debug)]
pub struct ConcreteRouteCalculator {
    context: ConductorContext,
}

impl ConcreteRouteCalculator {
    pub fn new(context: ConductorContext) -> Self {
        Self { context }
    }
    pub fn calc(
        &mut self,
        ship: &ship::MyShip,
        mut trade_route: ExtrapolatedTradeRoute,
        waypoints: &[database::Waypoint],
    ) -> ConcreteTradeRoute {
        let max_transport = ship.cargo.capacity;

        trade_route.data.max_trade_volume = max_transport.min(trade_route.data.max_trade_volume);
        trade_route.data.min_trade_volume = max_transport.min(trade_route.data.min_trade_volume);

        let (route_stats, route_to_stats) =
            self.calculate_route_statistics(ship, &trade_route, waypoints);

        let trip_stats = if true {
            self.calculate_reoccurring_trip_stats(
                ship,
                &trade_route,
                route_stats.distance,
                route_stats.fuel_cost,
                route_stats.travel_time,
            )
        } else {
            self.calculate_trip_stats(
                ship,
                &trade_route,
                route_stats.distance,
                route_stats.fuel_cost,
                route_stats.travel_time,
                route_to_stats.distance,
                route_to_stats.fuel_cost,
                route_to_stats.travel_time,
            )
        };

        self.create_concrete_route(trade_route, trip_stats)
    }

    fn calculate_route_statistics(
        &mut self,
        ship: &ship::MyShip,
        trade_route: &ExtrapolatedTradeRoute,
        waypoints: &[database::Waypoint],
    ) -> (RouteStats, RouteStats) {
        let waypoints = waypoints
            .iter()
            .map(|w| (w.symbol.clone(), w.clone()))
            .collect::<HashMap<_, _>>();
        let route = self.find_route(
            ship,
            &waypoints,
            &trade_route.route.sell.waypoint_symbol,
            &trade_route.route.purchase.waypoint_symbol,
        );

        let route_to = self.find_route(
            ship,
            &waypoints,
            &trade_route.route.sell.waypoint_symbol,
            &trade_route.route.purchase.waypoint_symbol,
        );

        (
            self.calculate_single_route_stats(ship, &route.unwrap()),
            self.calculate_single_route_stats(ship, &route_to.unwrap()),
        )
    }

    fn create_concrete_route(
        &self,
        trade_route: ExtrapolatedTradeRoute,
        trip_stats: TripStats,
    ) -> ConcreteTradeRoute {
        ConcreteTradeRoute {
            route: trade_route.route,
            data: trade_route.data,
            trip: trip_stats,
        }
    }

    fn find_route(
        &mut self,
        ship: &ship::MyShip,
        waypoints: &HashMap<String, database::Waypoint>,
        sell_wp_symbol: &str,
        purchase_wp_symbol: &str,
    ) -> Result<Vec<ship::autopilot::SimpleConnection>, crate::error::Error> {
        let pilot = ship
            .get_pathfinder(&self.context)
            .ok_or(crate::error::Error::General("NoAutopilot".to_string()))?
            .get_simple(waypoints.clone());
        pilot.find_route_system(sell_wp_symbol, purchase_wp_symbol)
    }

    fn calculate_reoccurring_trip_stats(
        &self,
        ship: &ship::MyShip,
        trade_route: &ExtrapolatedTradeRoute,
        total_distance: f64,
        total_fuel_cost: i32,
        total_travel_time: f64,
    ) -> TripStats {
        let trip_fuel_cost = (total_fuel_cost * 2) / 100 * CONFIG.trading.fuel_cost;

        let trip_volume = ship.cargo.capacity.min(
            (trade_route.data.min_trade_volume as f32 * CONFIG.trading.purchase_multiplier) as i32,
        );

        let trip_total_cost = trade_route.data.purchase_price * trip_volume + trip_fuel_cost;
        let trip_total_profit = trade_route.data.sell_price * trip_volume - trip_total_cost;

        let trip_per_hour = (total_travel_time * 1000.0 * 2.0)
            / (chrono::TimeDelta::hours(1).num_milliseconds()) as f64;

        let profit_per_hour = trip_total_profit as f64 / trip_per_hour;

        TripStats {
            time: total_travel_time * 2.0,
            fuel_cost: trip_fuel_cost,
            fuel_units: total_fuel_cost * 2,
            volume: trip_volume,
            total_cost: trip_total_cost,
            total_profit: trip_total_profit,
            trips_per_hour: trip_per_hour as f32,
            profit_per_hour: profit_per_hour as i32,
            ship_symbol: ship.symbol.clone(),
            distance: total_distance,
        }
    }

    fn calculate_trip_stats(
        &self,
        ship: &ship::MyShip,
        trade_route: &ExtrapolatedTradeRoute,
        total_distance: f64,
        total_fuel_cost: i32,
        total_travel_time: f64,
        total_distance_to: f64,
        total_fuel_cost_to: i32,
        total_travel_time_to: f64,
    ) -> TripStats {
        let trip_fuel_cost =
            (total_fuel_cost * total_fuel_cost_to) / 100 * CONFIG.trading.fuel_cost;

        let trip_volume = ship.cargo.capacity.min(
            (trade_route.data.min_trade_volume as f32 * CONFIG.trading.purchase_multiplier) as i32,
        );

        let trip_total_cost = trade_route.data.purchase_price * trip_volume + trip_fuel_cost;
        let trip_total_profit = trade_route.data.sell_price * trip_volume - trip_total_cost;

        let trip_per_hour = (total_travel_time * 1000.0 + total_travel_time_to * 1000.0)
            / (chrono::TimeDelta::hours(1).num_milliseconds()) as f64;

        let profit_per_hour = trip_total_profit as f64 / trip_per_hour;

        TripStats {
            ship_symbol: ship.symbol.clone(),
            trips_per_hour: trip_per_hour as f32,
            profit_per_hour: profit_per_hour as i32,
            fuel_units: total_fuel_cost + total_fuel_cost_to,
            fuel_cost: trip_fuel_cost,
            time: total_travel_time + total_travel_time_to,
            distance: total_distance + total_distance_to,
            volume: trip_volume,
            total_cost: trip_total_cost,
            total_profit: trip_total_profit,
        }
    }

    fn calculate_single_route_stats(
        &self,
        ship: &ship::MyShip,
        route: &[ship::autopilot::SimpleConnection],
    ) -> RouteStats {
        let route = ship
            .assemble_simple_route(route, CONFIG.trading.fuel_cost)
            .unwrap();

        RouteStats {
            fuel_cost: route.total_fuel_cost as i32,
            travel_time: route.total_travel_time,
            distance: route.total_distance,
        }
    }
}

#[derive(Debug)]
struct RouteStats {
    fuel_cost: i32,
    travel_time: f64,
    distance: f64,
}
