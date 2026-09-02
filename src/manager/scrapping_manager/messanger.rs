use super::message::ScrappingManagerMessage;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

#[derive(Debug, Clone)]
pub struct ScrappingManagerMessanger {
    pub sender: tokio::sync::mpsc::Sender<ScrappingManagerMessage>,
    busy: Arc<AtomicBool>,
    agent_scrapper_busy: Arc<AtomicBool>,
    agent_scrapper_count: Arc<AtomicU32>,
    agent_scrapper_active: Arc<AtomicBool>,
    system_scrapper_state: Arc<tokio::sync::RwLock<SystemScrapperState>>,
}

impl ScrappingManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<ScrappingManagerMessage>) -> Self {
        Self {
            sender,
            busy: Arc::new(AtomicBool::new(false)),
            agent_scrapper_busy: Arc::new(AtomicBool::new(false)),
            agent_scrapper_count: Arc::new(AtomicU32::new(0)),
            agent_scrapper_active: Arc::new(AtomicBool::new(false)),
            system_scrapper_state: Arc::new(tokio::sync::RwLock::new(
                SystemScrapperState::Inactive,
            )),
        }
    }

    #[tracing::instrument(skip(self, ship_clone), name = "ScrappingManagerMessanger::get_next", fields(ship = %ship_clone.symbol))]
    pub async fn get_next(
        &self,
        ship_clone: ship::MyShipCopy,
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
        ship_clone: ship::MyShipCopy,
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
        ship_clone: ship::MyShipCopy,
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
        ship_clone: ship::MyShipCopy,
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

    pub fn is_agent_scrapper_busy(&self) -> bool {
        self.agent_scrapper_busy.load(Ordering::Relaxed)
    }
    pub fn get_agent_scrapper_count(&self) -> u32 {
        self.agent_scrapper_count.load(Ordering::Relaxed)
    }
    pub fn is_agent_scrapper_active(&self) -> bool {
        self.agent_scrapper_active.load(Ordering::Relaxed)
    }
    pub fn set_agent_scrapper_busy(&self, state: bool) -> bool {
        self.agent_scrapper_busy.swap(state, Ordering::Relaxed)
    }
    pub fn increase_agent_scrapper_count(&self) -> bool {
        self.agent_scrapper_count.fetch_add(1, Ordering::Relaxed);
        true
    }
    pub fn set_agent_scrapper_active(&self, state: bool) -> bool {
        self.agent_scrapper_active.swap(state, Ordering::Relaxed)
    }

    pub async fn is_system_scrapper_active(&self) -> bool {
        let state = self.system_scrapper_state.read().await;
        !matches!(&*state, SystemScrapperState::Inactive)
    }

    pub async fn get_system_scrapper_state(&self) -> SystemScrapperState {
        let state = self.system_scrapper_state.read().await;
        (*state).clone()
    }

    pub async fn set_system_scrapper_state(&self, state: SystemScrapperState) -> bool {
        let mut current = self.system_scrapper_state.write().await;
        *current = state;
        true
    }
}

#[derive(Debug, Clone)]
pub enum SystemScrapperState {
    Inactive,
    ScrapSystems,
    ScrapWaypoints { total: u32, current: u32 },
    ScrapJumpGates,
}
