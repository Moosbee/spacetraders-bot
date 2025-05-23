use std::collections::HashMap;

use database::DatabaseConnector;
use log::debug;
use tokio::select;
use utils::WaypointCan;

use crate::{
    error::Result,
    manager::{
        fleet_manager::message::{Budget, Priority, RequestedShipType, RequiredShips},
        trade_manager::message::TradeMessage,
        Manager,
    },
    utils::ConductorContext,
};

use super::{
    messager::TradeManagerMessanger, routes_calculator::RouteCalculator,
    routes_tracker::RoutesTracker, TradeManagerMessage,
};

#[derive(Debug)]
pub struct TradeManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<TradeManagerMessage>,
    routes_tracker: RoutesTracker,
    calculator: RouteCalculator,
}

impl TradeManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<TradeManagerMessage>,
        TradeManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
        debug!("Created TradeManager channel");
        (receiver, TradeManagerMessanger::new(sender))
    }

    pub fn new(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<TradeManagerMessage>,
    ) -> Self {
        debug!("Created new TradeManager");
        Self {
            cancel_token,
            context: context.clone(),
            receiver,
            routes_tracker: RoutesTracker::default(),
            calculator: RouteCalculator::new(context),
        }
    }

    async fn run_trade_worker(&mut self) -> Result<()> {
        debug!("Starting TradeManager worker");
        while !self.cancel_token.is_cancelled() {
            let message: Option<TradeMessage> = select! {
                message = self.receiver.recv() => message,
                _ = self.cancel_token.cancelled() => None
            };
            debug!("Received message: {:?}", message);
            match message {
                Some(message) => {
                    self.handle_trade_message(message).await?;
                }
                None => {
                    debug!("No TradeManager more messages, exiting loop");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_trade_message(&mut self, message: TradeManagerMessage) -> Result<()> {
        debug!("Handling trade message: {:?}", message);
        match message {
            TradeMessage::RequestNextTradeRoute {
                ship_clone,
                callback,
            } => {
                let route = self.request_next_trade_route(ship_clone).await;
                debug!("Sending route: {:?}", route);
                let _send = callback.send(route);
            }
            TradeMessage::CompleteTradeRoute {
                trade_route,
                callback,
            } => {
                let route = self.complete_trade_route(trade_route).await;
                debug!("Sending route: {:?}", route);
                let _send = callback.send(route);
            }
            TradeMessage::GetPossibleTrades { callback } => {
                let trade_goods =
                    database::MarketTradeGood::get_last(&self.context.database_pool).await?;
                let market_trade =
                    database::MarketTrade::get_last(&self.context.database_pool).await?;

                let possible_trades = self
                    .calculator
                    .gen_all_possible_trades(&trade_goods, &market_trade);
                callback.send(possible_trades).map_err(|e| {
                    crate::error::Error::General(format!("Failed to send message: {:?}", e))
                })?
            }
        }
        Ok(())
    }

    pub fn get_required_ships(
        all_ships: &[ship::MyShip],
        all_systems_hashmap: &HashMap<String, HashMap<String, database::Waypoint>>,
        markets_per_ship: i64,
    ) -> Result<RequiredShips> {
        // let all_ships = context
        //     .ship_manager
        //     .get_all_clone()
        //     .await
        //     .into_values()
        //     .collect::<Vec<_>>();
        // let all_systems_hashmap: HashMap<String, HashMap<String, database::Waypoint>> = database::Waypoint::get_hash_map(&context.database_pool).await?;

        let mut systems: HashMap<String, Vec<String>> = HashMap::new();

        for s in all_ships {
            let system_str = match &s.role {
                database::ShipInfoRole::Transfer => match &s.status {
                    ship::ShipStatus::Transfer { system_symbol, .. } => {
                        system_symbol.clone().unwrap_or_default()
                    }
                    _ => s.nav.system_symbol.clone(),
                },
                _ => s.nav.system_symbol.clone(),
            };

            let is_scrapper = s.role == database::ShipInfoRole::Scraper
                || (s.role == database::ShipInfoRole::Transfer
                    && match &s.status {
                        ship::ShipStatus::Transfer { role, .. } => {
                            role == &Some(database::ShipInfoRole::Scraper)
                        }
                        _ => false,
                    });

            let is_trader = s.role == database::ShipInfoRole::Trader
                || (s.role == database::ShipInfoRole::Transfer
                    && match &s.status {
                        ship::ShipStatus::Transfer { role, .. } => {
                            role == &Some(database::ShipInfoRole::Trader)
                        }
                        _ => false,
                    });

            let system = systems.get_mut(&system_str);
            if let Some(system) = system {
                if is_trader {
                    system.push(s.symbol.clone());
                }
            } else if is_trader {
                systems.insert(system_str, vec![s.symbol.clone()]);
            } else if is_scrapper {
                systems.insert(system_str, vec![]);
            }
        }

        let mut required_ships = RequiredShips::new();

        for (system, ships) in systems {
            let waypoints = all_systems_hashmap
                .get(&system)
                .map(|s| s.values().filter(|w| w.is_marketplace()).count())
                .unwrap_or(0);
            let diff = ((waypoints as i64) / markets_per_ship) - (ships.len() as i64);
            if diff <= 0 {
                continue;
            };

            let mut sys_ships = (0..(diff as usize))
                .map(|_| {
                    (
                        RequestedShipType::Transporter,
                        Priority::Medium,
                        Budget::High,
                        database::ShipInfoRole::Trader,
                    )
                })
                .collect::<Vec<_>>();

            if ships.is_empty() && waypoints > 0 {
                sys_ships.pop();
                sys_ships.push((
                    RequestedShipType::Transporter,
                    Priority::High,
                    Budget::High,
                    database::ShipInfoRole::Trader,
                ));
            }

            let before = required_ships.ships.insert(system, sys_ships);
            if before.is_some() {
                log::warn!("Trading Ship contains ships");
            }
        }

        Ok(required_ships)
    }

    async fn request_next_trade_route(
        &mut self,
        ship_clone: ship::MyShip,
    ) -> Result<Option<database::TradeRoute>> {
        let unfinished_route =
            database::TradeRoute::get_unfinished(&self.context.database_pool).await?;
        let my_unfinished_routes = unfinished_route
            .iter()
            .filter(|r| r.ship_symbol == ship_clone.symbol)
            .collect::<Vec<_>>();

        let next_route = if !my_unfinished_routes.is_empty() {
            Some(my_unfinished_routes[0].clone())
        } else {
            let mode = { self.context.config.read().await.trade_mode };
            self.calculator
                .get_best_route(&ship_clone, &self.routes_tracker, mode)
                .await?
        };

        if next_route.is_none() {
            return Ok(None);
        }

        let next_route = next_route.unwrap();

        let next_route = self.record_trade_start(&next_route).await?;

        let done = self.routes_tracker.lock(&next_route.clone().into());

        if !done {
            return Err("Failed to lock route".into());
        }
        Ok(Some(next_route))
    }

    async fn complete_trade_route(
        &mut self,
        trade_route: database::TradeRoute,
    ) -> Result<database::TradeRoute> {
        let trade = self.complete_trade_record(trade_route).await?;
        self.routes_tracker.unlock(&trade.clone().into());
        Ok(trade)
    }

    async fn record_trade_start(
        &self,
        route: &database::TradeRoute,
    ) -> Result<database::TradeRoute> {
        if route.id == 0 {
            let id = database::TradeRoute::insert_new(&self.context.database_pool, route).await?;
            Ok(database::TradeRoute {
                id,
                ..route.clone()
            })
        } else {
            database::TradeRoute::insert(&self.context.database_pool, route).await?;
            Ok(route.clone())
        }
    }

    async fn complete_trade_record(
        &self,
        trade_route: database::TradeRoute,
    ) -> Result<database::TradeRoute> {
        let completed_route = trade_route.complete();
        database::TradeRoute::insert(&self.context.database_pool, &completed_route).await?;
        Ok(completed_route)
    }
}

impl Manager for TradeManager {
    fn run(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { self.run_trade_worker().await })
    }

    fn get_name(&self) -> &str {
        "TradeManager"
    }

    fn get_cancel_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancel_token
    }
}
