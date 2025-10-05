use tracing::instrument;

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

    #[instrument(skip(self, ship_clone), name = "ChartManagerMessanger::get_next", fields(ship = %ship_clone.symbol))]
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

    #[instrument(skip(self, waypoint_symbol), name = "ChartManagerMessanger::fail_chart", fields(waypoint = %waypoint_symbol))]
    pub async fn fail_chart(&self, waypoint_symbol: String) -> Result<(), crate::error::Error> {
        let message = ChartManagerMessage::Fail { waypoint_symbol };
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    #[instrument(skip(self, waypoint_symbol), name = "ChartManagerMessanger::complete_chart", fields(waypoint = %waypoint_symbol))]
    pub async fn complete_chart(&self, waypoint_symbol: String) -> Result<(), crate::error::Error> {
        let message = ChartManagerMessage::Success { waypoint_symbol };
        self.sender
            .send(message)
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))?;
        Ok(())
    }
}
