use std::collections::{HashMap, HashSet};

use database::{DatabaseConnectorAsync, PaginatedQuery};
use space_traders_client::models;
use tokio::select;
use tracing::debug;

use crate::{
    error::Result,
    manager::{
        Manager,
        trade_manager::{message::TradeMessage, trade_route_calculator},
    },
    utils::ConductorContext,
};

use super::{TradeManagerMessage, messager::TradeManagerMessanger, routes_tracker::RoutesTracker};

pub type TradeManagerReceiver = tokio::sync::mpsc::Receiver<TradeManagerMessage>;

#[derive(Debug)]
pub struct TradeManager {
    slow_cancel_token: tokio_util::sync::CancellationToken,
    fast_cancel_token: tokio_util::sync::CancellationToken,

    context: ConductorContext,
    receiver: TradeManagerReceiver,
    routes_tracker: RoutesTracker,
}

impl TradeManager {
    pub fn create() -> (TradeManagerReceiver, TradeManagerMessanger) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1024);
        debug!("Created TradeManager channel");
        (receiver, TradeManagerMessanger::new(sender))
    }

    pub async fn init(
        fast_cancel_token: tokio_util::sync::CancellationToken,
        slow_cancel_token: tokio_util::sync::CancellationToken,
        context: ConductorContext,
        receiver: TradeManagerReceiver,
    ) -> crate::error::Result<Self> {
        debug!("Created new TradeManager");

        let mut tracker = RoutesTracker::default();

        let unfinished_routes =
            database::TradeRoute::get_unfinished(&context.database_pool, PaginatedQuery::unpaged())
                .await?
                .items;

        for route in &unfinished_routes {
            let _ = tracker.lock(&route.clone().into());
        }

        Ok(Self {
            fast_cancel_token,
            slow_cancel_token,
            context: context.clone(),
            receiver,
            routes_tracker: tracker,
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
        let fast_cancel_token = self.fast_cancel_token.clone();
        select! {
            _ = fast_cancel_token.cancelled() => {
                tracing::info!("TradeManager fast cancel token triggered");
                return Ok(());
            },
            erg = self.run_trade_worker_loop() => return erg,
        }
    }

    async fn run_trade_worker_loop(&mut self) -> Result<()> {
        while !self.slow_cancel_token.is_cancelled() {
            let message: Option<TradeMessage> = select! {
                message = self.receiver.recv() => message,
                _ = self.slow_cancel_token.cancelled() => {
                    tracing::info!("TradeManager slow cancel token triggered");
                    None
                }
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
                trading_config,
                callback,
            } => {
                let route = self
                    .request_next_trade_route(ship_clone, trading_config)
                    .await;
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
            TradeMessage::GetLockedRoutes { callback } => {
                let routes = self.get_locked_routes().await;
                debug!("Sending routes: {:?}", routes);
                let _send = callback.send(routes);
            }
        }
        self.context.trade_manager.set_busy(false);

        Ok(())
    }

    async fn request_next_trade_route(
        &mut self,
        ship_clone: ship::MyShipCopy,
        trading_config: database::TradingFleetConfig,
    ) -> Result<Option<database::TradeRoute>> {
        let unfinished_route = database::TradeRoute::get_unfinished(
            &self.context.database_pool,
            PaginatedQuery::unpaged(),
        )
        .await?
        .items;
        let my_unfinished_routes = unfinished_route
            .iter()
            .filter(|r| r.ship_symbol == ship_clone.symbol)
            .collect::<Vec<_>>();

        let next_route_potential = if !my_unfinished_routes.is_empty() {
            (Some((my_unfinished_routes[0].clone(), 0)), true)
        } else {
            (
                self.get_best_ship_route(&ship_clone, trading_config)
                    .await?,
                false,
            )
        };

        if next_route_potential.0.is_none() {
            return Ok(None);
        }

        let (mut next_route, next_route_total_expense) = next_route_potential.0.unwrap();

        let done = self.routes_tracker.lock(&next_route.clone().into());

        if !done && !next_route_potential.1 {
            return Err("Failed to lock route".into());
        }
        if !done && next_route_potential.1 {
            tracing::warn!("Route was already locked, continuing");
        }

        if next_route.reserved_fund.is_none() {
            let reservation = self
                .context
                .budget_manager
                .reserve_funds_with_remain(
                    &self.context.database_pool,
                    next_route_total_expense,
                    1_000,
                )
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
                database::PaginatedQuery::unpaged(),
            )
            .await?
            .items;
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
            database::TradeRoute::upsert(&self.context.database_pool, route).await?;
            Ok(route.clone())
        }
    }

    async fn complete_trade_record(
        &self,
        trade_route: database::TradeRoute,
    ) -> Result<database::TradeRoute> {
        let completed_route = trade_route.complete();
        database::TradeRoute::upsert(&self.context.database_pool, &completed_route).await?;
        Ok(completed_route)
    }

    async fn get_best_ship_route(
        &self,
        ship_clone: &ship::RustShip<ship::status::ShipStatus>,
        trading_config: database::TradingFleetConfig,
    ) -> Result<Option<(database::TradeRoute, i64)>> {
        let trade_systems: Vec<String> =
            trade_route_calculator::get_trade_systems(&trading_config, ship_clone);

        let (trade_goods, market_trade) =
            trade_route_calculator::fetch_market_data(&self.context.database_pool, &trade_systems)
                .await?;

        let trade_route_candidates_all =
            trade_route_calculator::gen_all_trade_route_candidates(&trade_goods, &market_trade);

        let trade_route_candidates_filtered = trade_route_calculator::filter_trade_route_candidates(
            trade_route_candidates_all,
            &trading_config.market_blacklist,
        );

        let config = self.context.config.read().await.clone();

        let trade_route_proposals = trade_route_calculator::gen_trade_route_proposals(
            &self.context.database_pool,
            trade_route_candidates_filtered,
            &trade_systems,
            &ship_clone.get_nav_stats(),
            trading_config.purchase_multiplier,
            config.default_purchase_price,
            config.default_sell_price,
        )
        .await?;

        let spendable = self
            .context
            .budget_manager
            .get_spendable_funds_with_remain(10000)
            .await;

        let trade_route_proposal = trade_route_calculator::get_best_trade_route_proposal(
            trade_route_proposals,
            |trp| {
                trade_route_calculator::filter_trade_route_proposal(trp, &trading_config)
                    && !self.routes_tracker.is_locked(&(trp.into()))
                    && (trp.total_cost as i64) <= spendable
            },
            |trp1, trp2| {
                trade_route_calculator::sort_trade_route_proposal(
                    trp1,
                    trp2,
                    &trading_config,
                    &self.context.supply_chain_mapping,
                )
                .unwrap_or(std::cmp::Ordering::Equal)
            },
        );

        let expenses = trade_route_proposal
            .as_ref()
            .map(|trp| trp.total_cost as i64)
            .unwrap_or(0);

        let mut trade_route_db: Option<database::TradeRoute> = trade_route_proposal.map(Into::into);

        if let Some(v) = trade_route_db.as_mut() {
            v.trade_mode = trading_config.trade_mode;
            v.ship_symbol = ship_clone.symbol.to_string();
            v.assignment_id = ship_clone
                .status
                .temp_assignment_id
                .or(ship_clone.status.assignment_id);
            v.fleet_id = ship_clone
                .status
                .temp_fleet_id
                .or(ship_clone.status.fleet_id);
        }

        Ok(trade_route_db.map(|tr_db| (tr_db, expenses)))
    }

    async fn get_locked_routes(&self) -> HashSet<super::routes_tracker::RouteLock> {
        self.routes_tracker.get_locked_routes()
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
        &self.slow_cancel_token
    }
}
