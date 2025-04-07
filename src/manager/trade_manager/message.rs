use crate::error::Result;
use crate::manager::fleet_manager::message::RequiredShips;

use super::routes::PossibleTradeRoute;

#[derive(Debug)]
pub enum TradeMessage {
    RequestNextTradeRoute {
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<database::TradeRoute>>,
    },
    CompleteTradeRoute {
        trade_route: database::TradeRoute,
        callback: tokio::sync::oneshot::Sender<Result<database::TradeRoute>>,
    },
    GetShips {
        callback: tokio::sync::oneshot::Sender<RequiredShips>,
    },
    GetPossibleTrades {
        callback: tokio::sync::oneshot::Sender<Vec<PossibleTradeRoute>>,
    },
}

pub type TradeManagerMessage = TradeMessage;
