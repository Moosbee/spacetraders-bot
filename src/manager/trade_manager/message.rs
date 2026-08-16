use std::collections::{HashMap, HashSet};

use crate::error::Result;

#[derive(Debug)]
pub enum TradeMessage {
    RequestNextTradeRoute {
        ship_clone: ship::MyShipCopy,
        trading_config: database::TradingFleetConfig,
        callback: tokio::sync::oneshot::Sender<Result<Option<database::TradeRoute>>>,
    },
    CompleteTradeRoute {
        trade_route: database::TradeRoute,
        callback: tokio::sync::oneshot::Sender<Result<database::TradeRoute>>,
    },
    GetLockedRoutes {
        callback: tokio::sync::oneshot::Sender<HashSet<super::routes_tracker::RouteLock>>,
    },
}

pub type TradeManagerMessage = TradeMessage;
