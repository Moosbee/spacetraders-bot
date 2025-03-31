use log::debug;

use crate::{error::Error, manager::fleet_manager::message::RequiredShips, ship, sql};

use super::TradeManagerMessage;

#[derive(Debug, Clone)]
pub struct TradeManagerMessanger {
    sender: tokio::sync::mpsc::Sender<TradeManagerMessage>,
}

impl TradeManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<super::message::TradeMessage>) -> Self {
        Self { sender }
    }

    pub async fn get_route(&self, ship: &ship::MyShip) -> Result<sql::TradeRoute, Error> {
        debug!("Requesting next trade route for ship {}", ship.symbol);
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let message = TradeManagerMessage::RequestNextTradeRoute {
            ship_clone: ship.clone(),
            callback: sender,
        };

        self.sender
            .send(message)
            .await
            .map_err(|e| Error::General(format!("Failed to send message: {}", e)))?;

        debug!("Requested next trade route for ship {}", ship.symbol);

        let resp = receiver
            .await
            .map_err(|e| Error::General(format!("Failed to get trade get message: {}", e)))?;

        debug!("Received trade route for ship {}: {:?}", ship.symbol, resp);
        resp
    }

    pub async fn complete_trade(
        &self,
        trade_route: &sql::TradeRoute,
    ) -> Result<sql::TradeRoute, Error> {
        debug!("Completing trade route: {:?}", trade_route);
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

        debug!("Completed trade route: {:?}", resp);
        resp
    }

    pub async fn get_ships(&self) -> Result<RequiredShips, crate::error::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(TradeManagerMessage::GetShips { callback: tx })
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))
            .unwrap();
        rx.await
            .map_err(|e| crate::error::Error::General(format!("Failed to receive message: {}", e)))
    }
}
