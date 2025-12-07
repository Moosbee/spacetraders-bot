use std::sync::{atomic::AtomicBool, Arc};

use crate::error::Error;

use super::TradeManagerMessage;

#[derive(Debug, Clone)]
pub struct TradeManagerMessanger {
    sender: tokio::sync::mpsc::Sender<TradeManagerMessage>,
    busy: Arc<AtomicBool>,
}

impl TradeManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<super::message::TradeMessage>) -> Self {
        Self {
            sender,
            busy: Arc::new(AtomicBool::new(false)),
        }
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

    pub fn is_busy(&self) -> bool {
        self.busy.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn set_busy(&self, busy: bool) -> bool {
        self.busy.swap(busy, std::sync::atomic::Ordering::Relaxed)
    }
    pub fn get_channel_state(&self) -> crate::utils::ChannelInfo {
        let state = if self.sender.is_closed() {
            crate::utils::ChannelState::Closed
        } else {
            crate::utils::ChannelState::Open
        };

        let max_capacity = self.sender.max_capacity();
        let free_capacity = self.sender.capacity();
        let used_capacity = max_capacity - free_capacity;

        crate::utils::ChannelInfo {
            state,
            total_capacity: max_capacity,
            used_capacity,
            free_capacity,
        }
    }
}
