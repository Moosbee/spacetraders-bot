use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use log::{debug, info};
use space_traders_client::models::{self, waypoint, Ship};

use crate::{
    ship::{self, nav_models::Cache, MyShip},
    sql::{self, DatabaseConnector},
    workers::{
        trading::t_types::ConcreteTradeRoute,
        types::{Conductor, ConductorContext},
    },
};

use super::{routes_track_keeper::RoutesTrackKeeper, t_types::PossibleTradeRoute};

const FUEL_COST: i32 = 72;
const PURCHASE_MULTIPLIER: f32 = 2.0;

const BLACKLIST: [models::TradeSymbol; 2] = [
    models::TradeSymbol::AdvancedCircuitry,
    models::TradeSymbol::FabMats,
];

#[derive(Debug, Clone)]
pub struct TradingFleet {
    context: ConductorContext,
    running_routes: Arc<RoutesTrackKeeper>,
    cache: Arc<RwLock<Cache>>,
}

impl TradingFleet {
    #[allow(dead_code)]
    pub fn new_box(context: ConductorContext) -> Box<Self> {
        Box::new(TradingFleet {
            context,
            running_routes: Arc::new(RoutesTrackKeeper::default()),
            cache: Arc::new(RwLock::new(Cache::default())),
        })
    }

    async fn run_trade_worker(&self) -> anyhow::Result<()> {
        info!("Starting trading workers");

        let ships = self
            .context
            .ship_roles
            .iter()
            .filter(|(_, role)| **role == ship::models::Role::Trader)
            .map(|(symbol, _)| symbol.clone())
            .collect::<Vec<_>>();

        let trade_goods: Vec<sql::MarketTradeGood> =
            sql::MarketTradeGood::get_last(&self.context.database_pool).await?;
        let routes = TradingFleet::calc_possible_trade_routes(trade_goods)
            .into_iter()
            .filter(|route| !BLACKLIST.contains(&route.symbol))
            .collect::<Vec<_>>();
        // routes.sort();
        // for route in routes {
        //     info!("Route: {}", route);
        // }

        info!("Possible routes: {}", routes.len());

        let mut concrete_routes: Vec<ConcreteTradeRoute> = routes
            .iter()
            // .filter(|route| route.profit > 0)
            .flat_map(|route| {
                ships
                    .iter()
                    .map(|ship| {
                        let mut this_cache = self.cache.write().unwrap();
                        let ship = self.context.my_ships.get(ship).unwrap();

                        self.calc_concrete_trade_route(&*ship, route.clone(), &mut *this_cache)
                    })
                    .collect::<Vec<ConcreteTradeRoute>>()
            })
            .collect();

        concrete_routes.sort_by(|a, b| a.partial_cmp(b).unwrap());

        info!("Concrete routes: {}", concrete_routes.len());

        for route in concrete_routes {
            info!("Route: {}", route);
        }

        let mut handles = Vec::new();

        for ship in ships {
            let fleet = self.clone();
            let handle = tokio::spawn(async move { fleet.run_trade_ship_worker(ship).await });
            handles.push(handle);
        }

        info!("Waiting for trading workers to finish");

        for handle in handles {
            let _ = handle.await.unwrap().unwrap();
        }

        info!("Trading workers done");

        Ok(())
    }

    async fn run_trade_ship_worker(&self, ship_symbol: String) -> anyhow::Result<()> {
        let mut ship = self.context.my_ships.get_mut(&ship_symbol).unwrap();

        if ship.cargo.units == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(
                1000 + rand::random::<u64>() % 1000,
            ))
            .await;
        } else {
            self.finish_cargo_trade(&mut ship).await?;
        }

        Ok(())
    }

    async fn finish_cargo_trade(&self, ship: &mut ship::MyShip) -> anyhow::Result<()> {
        let cargo = ship.cargo.inventory.clone();
        for cargo_item in cargo {
            let trade_goods: Vec<sql::MarketTradeGood> =
                sql::MarketTradeGood::get_last(&self.context.database_pool).await?;
            let mut routes = TradingFleet::calc_possible_trade_routes(trade_goods)
                .into_iter()
                .filter(|route| &route.symbol == &cargo_item.0)
                .collect::<Vec<_>>();

            routes.sort_by_key(|route| route.sell_price);
            if let Some(trade_route) = routes.pop() {
                let concrete = {
                    let mut this_cache = self.cache.write().unwrap();
                    self.calc_concrete_trade_route(ship, trade_route, &mut this_cache)
                };
                if ship.cargo.has(&cargo_item.0) {
                    self.process_trade_route(&mut *ship, concrete).await?
                }
            } else {
                return Err(anyhow::anyhow!("No routes found for {}", cargo_item.0));
            }
        }
        Ok(())
    }

    async fn process_trade_route(
        &self,
        ship: &mut ship::MyShip,
        concrete_route: ConcreteTradeRoute,
    ) -> anyhow::Result<()> {
        if self
            .running_routes
            .is_locked(&concrete_route.clone().into())
        {
            return Err(anyhow::anyhow!("Route is locked"));
        }

        self.running_routes.lock(&concrete_route.clone().into())?;

        info!("Processing route: {}", concrete_route);

        let sql_route: sql::TradeRoute = concrete_route.clone().into();

        let id = sql::TradeRoute::insert_id(&self.context.database_pool, &sql_route).await?;

        let waypoints = {
            self.context
                .all_waypoints
                .get(&ship.nav.system_symbol)
                .unwrap()
                .clone()
        };

        if !ship.cargo.has(&concrete_route.symbol) {
            ship.nav_to(
                &concrete_route.purchase_wp_symbol,
                true,
                &waypoints,
                &self.context.api,
                self.context.database_pool.clone(),
                sql::TransactionReason::TradeRoute(id),
            )
            .await?;

            ship.ensure_docked(&self.context.api).await?;

            ship.purchase_cargo(
                &self.context.api,
                concrete_route.symbol,
                concrete_route.trip_units,
                &self.context.database_pool,
                sql::TransactionReason::TradeRoute(id),
            )
            .await?;
        }

        ship.nav_to(
            &concrete_route.sell_wp_symbol,
            true,
            &waypoints,
            &self.context.api,
            self.context.database_pool.clone(),
            sql::TransactionReason::TradeRoute(id),
        )
        .await?;

        ship.ensure_docked(&self.context.api).await?;

        let cargo_volume = ship.cargo.get_amount(&concrete_route.symbol);
        ship.sell_cargo(
            &self.context.api,
            concrete_route.symbol,
            cargo_volume,
            &self.context.database_pool,
            sql::TransactionReason::TradeRoute(id),
        )
        .await?;

        let fin_route = sql_route.complete();

        sql::TradeRoute::insert(&self.context.database_pool, &fin_route).await?;

        self.running_routes.unlock(&fin_route.into());
        Ok(())
    }

    fn calc_possible_trade_routes(
        trade_goods: Vec<sql::MarketTradeGood>,
    ) -> Vec<PossibleTradeRoute> {
        let mut trades: HashMap<
            models::TradeSymbol,
            (Vec<sql::MarketTradeGood>, Vec<sql::MarketTradeGood>),
        > = HashMap::new();
        for t in trade_goods {
            debug!("Trade: {:?}", t);
            let v = trades.get(&t.symbol);

            let mut trade: (Vec<_>, Vec<_>) = v.unwrap_or(&(Vec::new(), Vec::new())).clone();
            trade.0.push(t.clone());
            trade.1.push(t.clone());
            trades.insert(t.symbol, trade);
        }

        trades
            .iter()
            .flat_map(|(trade_good, (exports, imports))| {
                let exports: Vec<_> = exports
                    .iter()
                    .map(|export| {
                        let imports: Vec<_> = imports
                            .iter()
                            .map(|import| {
                                // (trade_good.clone(), export.clone(), import.clone())

                                PossibleTradeRoute {
                                    symbol: *trade_good,
                                    export: export.clone(),
                                    import: import.clone(),
                                    min_trade_volume: export.trade_volume.min(import.trade_volume),
                                    max_trade_volume: export.trade_volume.max(import.trade_volume),
                                    purchase_wp_symbol: export.waypoint_symbol.clone(),
                                    sell_wp_symbol: import.waypoint_symbol.clone(),
                                    purchase_price: export.purchase_price,
                                    sell_price: import.sell_price,
                                    profit: import.sell_price - export.purchase_price,
                                }
                            })
                            .collect();
                        imports
                    })
                    .collect();
                exports
            })
            .flatten()
            .collect()
    }

    fn calc_concrete_trade_route(
        &self,
        ship: &ship::MyShip,
        trade_route: PossibleTradeRoute,
        cache: &mut Cache,
    ) -> ConcreteTradeRoute {
        info!("Calculating trade route: {}", trade_route);
        let waypoints = &self
            .context
            .all_waypoints
            .get(&ship.nav.system_symbol)
            .unwrap()
            .clone();

        let route = ship.find_route_cached(
            waypoints,
            trade_route.sell_wp_symbol.clone(),
            trade_route.purchase_wp_symbol.clone(),
            &ship::nav_models::NavMode::BurnAndCruiseAndDrift,
            true,
            cache,
        );

        let (_stats, _total_distance, total_fuel_cost, total_travel_time): (
            Vec<ship::nav_models::ConnectionDetails>,
            f64,
            i32,
            chrono::TimeDelta,
        ) = ship::stats::calc_route_stats(waypoints, &route.unwrap(), ship.engine_speed);

        let trip_fuel_cost = (total_fuel_cost * 2) / 100 * FUEL_COST;

        let trip_volume = ship
            .cargo
            .capacity
            .min((trade_route.min_trade_volume as f32 * PURCHASE_MULTIPLIER) as i32);

        let trip_total_cost = trade_route.purchase_price * trip_volume + trip_fuel_cost;
        let trip_total_profit = trade_route.sell_price * trip_volume - trip_total_cost;

        let trip_per_hour = (total_travel_time.num_milliseconds() * 2) as f64
            / (chrono::TimeDelta::hours(1).num_milliseconds()) as f64;

        let profit_per_hour = trip_total_profit as f64 / trip_per_hour;

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
}

impl Conductor for TradingFleet {
    fn run(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_trade_worker().await })
    }

    fn get_name(&self) -> String {
        "TradingFleet".to_string()
    }
}
