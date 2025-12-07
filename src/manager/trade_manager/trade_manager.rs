use database::DatabaseConnector;
use space_traders_client::models;
use tokio::select;
use tracing::debug;

use crate::{
    error::Result,
    manager::{trade_manager::message::TradeMessage, Manager},
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

    pub async fn init(
        cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: tokio::sync::mpsc::Receiver<TradeManagerMessage>,
    ) -> crate::error::Result<Self> {
        debug!("Created new TradeManager");

        let mut tracker = RoutesTracker::default();

        let unfinished_routes =
            database::TradeRoute::get_unfinished(&context.database_pool).await?;

        for route in &unfinished_routes {
            let _ = tracker.lock(&route.clone().into());
        }

        Ok(Self {
            cancel_token,
            context: context.clone(),
            receiver,
            routes_tracker: tracker,
            calculator: RouteCalculator::new(context),
        })
    }

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::trade_manager::trade_manager_worker",
        skip(self),
        err(Debug)
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "spacetraders::manager::trade_manager::trade_manager_handle_trade_message",
        skip(self),
        err(Debug)
    )]
    async fn handle_trade_message(&mut self, message: TradeManagerMessage) -> Result<()> {
        self.context.trade_manager.set_busy(true);
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
        self.context.trade_manager.set_busy(false);

        Ok(())
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

        let next_route_potential = if !my_unfinished_routes.is_empty() {
            (Some(my_unfinished_routes[0].clone()), true)
        } else {
            let mode = { self.context.config.read().await.trade_mode };
            (
                self.calculator
                    .get_best_route(&ship_clone, &self.routes_tracker, mode)
                    .await?,
                false,
            )
        };

        if next_route_potential.0.is_none() {
            return Ok(None);
        }

        let mut next_route = next_route_potential.0.unwrap();

        let done = self.routes_tracker.lock(&next_route.clone().into());

        if !done && !next_route_potential.1 {
            return Err("Failed to lock route".into());
        }
        if !done && next_route_potential.1 {
            tracing::warn!("Route was already locked, continuing");
        }

        if next_route.reserved_fund.is_none() {
            let total_expense =
                (next_route.predicted_purchase_price * next_route.trade_volume) as i64;

            let reservation = self
                .context
                .budget_manager
                .reserve_funds_with_remain(&self.context.database_pool, total_expense, 1_000)
                .await?;

            next_route.reserved_fund = Some(reservation.id);
        }

        let next_route = self.record_trade_start(&next_route).await?;

        Ok(Some(next_route))
    }

    async fn complete_trade_route(
        &mut self,
        trade_route: database::TradeRoute,
    ) -> Result<database::TradeRoute> {
        let trade = self.complete_trade_record(trade_route).await?;

        if let Some(reservation_id) = trade.reserved_fund {
            let transactions = database::MarketTransaction::get_by_reason(
                &self.context.database_pool,
                database::TransactionReason::TradeRoute(trade.id),
            )
            .await?;
            let actual_amount = transactions
                .iter()
                .filter(|t| t.r#type == models::market_transaction::Type::Purchase)
                .map(|t| t.total_price as i64)
                .sum();

            self.context
                .budget_manager
                .complete_use_reservation(
                    &self.context.database_pool,
                    reservation_id,
                    actual_amount,
                )
                .await?;
        }

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
