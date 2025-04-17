use crate::error::Result;

use super::routes::PossibleTradeRoute;

#[derive(Debug)]
pub enum TradeMessage {
    RequestNextTradeRoute {
        ship_clone: ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<Option<database::TradeRoute>>>,
    },
    CompleteTradeRoute {
        trade_route: database::TradeRoute,
        callback: tokio::sync::oneshot::Sender<Result<database::TradeRoute>>,
    },
    GetPossibleTrades {
        callback: tokio::sync::oneshot::Sender<Vec<PossibleTradeRoute>>,
    },
}

pub type TradeManagerMessage = TradeMessage;
