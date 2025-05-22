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
        let erg = self
            .sender
            .send_timeout(
                FleetManagerMessage::ScrapperAtShipyard {
                    waypoint_symbol,
                    ship_symbol,
                    callback: sender,
                },
                tokio::time::Duration::from_millis(5000),
            )
            .await;
        if let Err(e) = erg {
            match e {
                tokio::sync::mpsc::error::SendTimeoutError::Timeout(_e) => return Ok(()),
                tokio::sync::mpsc::error::SendTimeoutError::Closed(_e) => {
                    // return Err(crate::error::Error::General("Channel closed".to_string()))
                    return Ok(());
                }
            }
        }
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
