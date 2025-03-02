use log::debug;
use tokio::select;

use crate::{
    error::Result,
    manager::Manager,
    ship::nav_models::Cache,
    sql::{self, DatabaseConnector},
    types::ConductorContext,
};

use super::{routes_calculator::RouteCalculator, routes_tracker::RoutesTracker};

#[derive(Debug)]
pub enum TradeMessage {
    RequestNextTradeRoute {
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<sql::TradeRoute>>,
    },
    CompleteTradeRoute {
        trade_route: sql::TradeRoute,
        callback: tokio::sync::oneshot::Sender<Result<sql::TradeRoute>>,
    },
}

pub type TradeManagerMessage = TradeMessage;

#[derive(Debug)]
pub struct TradeManager {
    cancel_token: tokio_util::sync::CancellationToken,
    context: ConductorContext,
    receiver: tokio::sync::mpsc::Receiver<TradeManagerMessage>,
    routes_tracker: RoutesTracker,
    calculator: RouteCalculator,
}

#[derive(Debug, Clone)]
pub struct TradeManagerMessanger {
    pub sender: tokio::sync::mpsc::Sender<TradeManagerMessage>,
}

impl TradeManager {
    pub fn create() -> (
        tokio::sync::mpsc::Receiver<TradeManagerMessage>,
        TradeManagerMessanger,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
        debug!("Created TradeManager channel");
        (receiver, TradeManagerMessanger { sender })
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
            calculator: RouteCalculator::new(context, Cache::default()),
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
                None => break,
            }
        }

        Ok(())
    }

    async fn handle_trade_message(&mut self, message: TradeManagerMessage) -> Result<()> {
        debug!("Handling message: {:?}", message);
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
        }
        Ok(())
    }

    async fn request_next_trade_route(
        &mut self,
        ship_clone: crate::ship::MyShip,
    ) -> Result<sql::TradeRoute> {
        let unfinished_route = sql::TradeRoute::get_unfinished(&self.context.database_pool).await?;
        let my_unfinished_routes = unfinished_route
            .iter()
            .filter(|r| r.ship_symbol == ship_clone.symbol)
            .collect::<Vec<_>>();

        let next_route = if !my_unfinished_routes.is_empty() {
            my_unfinished_routes[0].clone()
        } else {
            self.calculator
                .get_best_route(&ship_clone, &self.routes_tracker)
                .await?
        };

        let next_route = self.record_trade_start(&next_route).await?;

        let done = self.routes_tracker.lock(&next_route.clone().into());

        if !done {
            return Err("Failed to lock route".into());
        }
        return Ok(next_route);
    }

    async fn complete_trade_route(
        &mut self,
        trade_route: sql::TradeRoute,
    ) -> Result<sql::TradeRoute> {
        let trade = self.complete_trade_record(trade_route).await?;
        self.routes_tracker.unlock(&trade.clone().into());
        Ok(trade)
    }

    async fn record_trade_start(&self, route: &sql::TradeRoute) -> Result<sql::TradeRoute> {
        if route.id == 0 {
            let id = sql::TradeRoute::insert_new(&self.context.database_pool, route).await?;
            Ok(sql::TradeRoute {
                id,
                ..route.clone()
            })
        } else {
            sql::TradeRoute::insert(&self.context.database_pool, route).await?;
            Ok(route.clone())
        }
    }

    async fn complete_trade_record(&self, trade_route: sql::TradeRoute) -> Result<sql::TradeRoute> {
        let completed_route = trade_route.complete();
        sql::TradeRoute::insert(&self.context.database_pool, &completed_route).await?;
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
