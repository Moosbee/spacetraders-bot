use crate::error::Result;
use crate::manager::fleet_manager::message::RequiredShips;
use crate::sql;

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
    GetShips {
        callback: tokio::sync::oneshot::Sender<RequiredShips>,
    },
}

pub type TradeManagerMessage = TradeMessage;
