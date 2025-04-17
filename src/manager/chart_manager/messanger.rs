use crate::manager::fleet_manager::message::RequiredShips;

use super::{
    messages::{ChartManagerMessage, NextChartResp},
    ChartManager,
};

#[derive(Debug, Clone)]
pub struct ChartManagerMessanger {
    sender: tokio::sync::mpsc::Sender<ChartManagerMessage>,
}

impl ChartManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<ChartManagerMessage>) -> Self {
        Self { sender }
    }

    pub async fn get_next(
        &self,
        ship_clone: ship::MyShip,
    ) -> Result<NextChartResp, crate::error::Error> {
        let (sender, callback) = tokio::sync::oneshot::channel();

        let message = ChartManagerMessage::Next {
            ship_clone,
            callback: sender,
        };
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;

        let resp = callback.await.map_err(|e| {
            crate::error::Error::General(format!("Failed to receive message: {}", e))
        })??;

        Ok(resp)
    }

    pub async fn fail_chart(&self, waypoint_symbol: String) -> Result<(), crate::error::Error> {
        let message = ChartManagerMessage::Fail { waypoint_symbol };
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    pub async fn complete_chart(&self, waypoint_symbol: String) -> Result<(), crate::error::Error> {
        let message = ChartManagerMessage::Success { waypoint_symbol };
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    pub async fn get_ships(
        &self,
        context: &crate::utils::ConductorContext,
    ) -> Result<RequiredShips, crate::error::Error> {
        ChartManager::get_required_ships(context).await
    }
}
