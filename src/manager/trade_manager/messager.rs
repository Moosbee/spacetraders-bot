
use crate::error::Error;

use super::{routes::PossibleTradeRoute, TradeManagerMessage};

#[derive(Debug, Clone)]
pub struct TradeManagerMessanger {
    sender: tokio::sync::mpsc::Sender<TradeManagerMessage>,
}

impl TradeManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<super::message::TradeMessage>) -> Self {
        Self { sender }
    }

    pub async fn get_route(
        &self,
        ship: &ship::MyShip,
    ) -> Result<Option<database::TradeRoute>, Error> {
    tracing::debug!(ship_symbol = %ship.symbol, "Requesting next trade route for ship");
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let message = TradeManagerMessage::RequestNextTradeRoute {
            ship_clone: ship.clone(),
            callback: sender,
        };

        self.sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

    tracing::debug!(ship_symbol = %ship.symbol, "Requested next trade route for ship");

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get trade get message: {}", e)))?;

    tracing::debug!(ship_symbol = %ship.symbol, resp = ?resp, "Received trade route for ship");
        resp
    }

    pub async fn complete_trade(
        &self,
        trade_route: &database::TradeRoute,
    ) -> Result<database::TradeRoute, Error> {
    tracing::debug!(trade_route_id = %trade_route.id, "Completing trade route");
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let message = TradeManagerMessage::CompleteTradeRoute {
            trade_route: trade_route.clone(),
            callback: sender,
        };

        self.sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get trade complete message: {}", e)))?;

    tracing::debug!(resp = ?resp, "Completed trade route");
        resp
    }

    pub(crate) async fn get_trades(&self) -> Result<Vec<PossibleTradeRoute>, crate::error::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(TradeManagerMessage::GetPossibleTrades { callback: tx })
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))
            .unwrap();
        rx.await
            .map_err(|e| crate::error::Error::General(format!("Failed to receive message: {}", e)))
    }
}
