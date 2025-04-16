use super::message::FleetManagerMessage;

#[derive(Debug, Clone)]
pub struct FleetManagerMessanger {
    sender: tokio::sync::mpsc::Sender<FleetManagerMessage>,
}

impl FleetManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<FleetManagerMessage>) -> Self {
        Self { sender }
    }

    pub async fn at_shipyard(
        &self,
        waypoint_symbol: String,
        ship_symbol: String,
    ) -> Result<(), crate::error::Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.sender
            .send(FleetManagerMessage::ScrapperAtShipyard {
                waypoint_symbol,
                ship_symbol,
                callback: sender,
            })
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;
        let _erg = receiver
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;

        Ok(())
    }

    pub async fn ship_arrived(
        &self,
        waypoint_symbol: String,
        ship_symbol: String,
    ) -> Result<(), crate::error::Error> {
        self.sender
            .send(FleetManagerMessage::ShipArrived {
                waypoint_symbol,
                ship_symbol,
            })
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;
        Ok(())
    }

    pub async fn get_transfer(
        &self,
        ship_clone: ship::MyShip,
    ) -> Result<database::ShipTransfer, crate::error::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(FleetManagerMessage::GetTransfer {
                ship_clone,
                callback: tx,
            })
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to send message: {}", e)))
            .unwrap();
        rx.await
            .map_err(|e| crate::error::Error::General(format!("Failed to receive message: {}", e)))
    }
}
