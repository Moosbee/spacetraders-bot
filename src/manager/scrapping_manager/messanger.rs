use super::message::ScrappingManagerMessage;
use std::sync::{atomic::AtomicBool, Arc};

#[derive(Debug, Clone)]
pub struct ScrappingManagerMessanger {
    pub sender: tokio::sync::mpsc::Sender<ScrappingManagerMessage>,
    busy: Arc<AtomicBool>,
}

impl ScrappingManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<ScrappingManagerMessage>) -> Self {
        Self {
            sender,
            busy: Arc::new(AtomicBool::new(false)),
        }
    }

    #[tracing::instrument(skip(self, ship_clone), name = "ScrappingManagerMessanger::get_next", fields(ship = %ship_clone.symbol))]
    pub async fn get_next(
        &self,
        ship_clone: ship::MyShip,
    ) -> Result<super::message::ScrapResponse, crate::error::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(ScrappingManagerMessage::Next {
                ship_clone,
                callback: tx,
            })
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;
        rx.await
            .map_err(|e| crate::error::Error::General(format!("Failed to receive message: {}", e)))
    }

    #[tracing::instrument(skip(self, ship_clone, waypoint_symbol), name = "ScrappingManagerMessanger::fail", fields(ship = %ship_clone.symbol, waypoint = %waypoint_symbol))]
    pub async fn fail(
        &self,
        ship_clone: ship::MyShip,
        waypoint_symbol: String,
    ) -> Result<(), crate::error::Error> {
        self.sender
            .send(ScrappingManagerMessage::Fail {
                ship_clone,
                waypoint_symbol,
            })
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ship_clone, waypoint_symbol), name = "ScrappingManagerMessanger::complete", fields(ship = %ship_clone.symbol, waypoint = %waypoint_symbol))]
    pub async fn complete(
        &self,
        ship_clone: ship::MyShip,
        waypoint_symbol: String,
    ) -> Result<(), crate::error::Error> {
        self.sender
            .send(ScrappingManagerMessage::Complete {
                ship_clone,
                waypoint_symbol,
            })
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ship_clone), name = "ScrappingManagerMessanger::get_info", fields(ship = %ship_clone.symbol))]
    pub(crate) async fn get_info(
        &self,
        ship_clone: ship::MyShip,
    ) -> Result<Vec<(String, chrono::DateTime<chrono::Utc>)>, crate::error::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(ScrappingManagerMessage::GetAll {
                callback: tx,
                ship_clone,
            })
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))
            .unwrap();
        rx.await
            .map_err(|e| crate::error::Error::General(format!("Failed to receive message: {}", e)))
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
