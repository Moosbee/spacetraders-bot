use crate::manager::fleet_manager::message::RequiredShips;

use super::message::ScrappingManagerMessage;

#[derive(Debug, Clone)]
pub struct ScrappingManagerMessanger {
    pub sender: tokio::sync::mpsc::Sender<ScrappingManagerMessage>,
}

impl ScrappingManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<ScrappingManagerMessage>) -> Self {
        Self { sender }
    }

    pub async fn get_next(
        &self,
        ship_clone: crate::ship::MyShip,
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

    pub async fn fail(
        &self,
        ship_clone: crate::ship::MyShip,
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

    pub async fn complete(
        &self,
        ship_clone: crate::ship::MyShip,
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

    pub(crate) async fn get_info(
        &self,
        ship_clone: crate::ship::MyShip,
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

    pub async fn get_ships(&self) -> Result<RequiredShips, crate::error::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(ScrappingManagerMessage::GetShips { callback: tx })
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))
            .unwrap();
        rx.await
            .map_err(|e| crate::error::Error::General(format!("Failed to receive message: {}", e)))
    }
}
