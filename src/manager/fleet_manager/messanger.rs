use super::message::FleetManagerMessage;

#[derive(Debug, Clone)]
pub struct FleetManagerMessanger {
    sender: tokio::sync::mpsc::Sender<FleetManagerMessage>,
}

impl FleetManagerMessanger {
    pub fn new(sender: tokio::sync::mpsc::Sender<FleetManagerMessage>) -> Self {
        Self { sender }
    }

    #[tracing::instrument(skip(self, waypoint_symbol, ship_symbol), name = "FleetManagerMessanger::at_shipyard", fields(waypoint = %waypoint_symbol, ship = %ship_symbol))]
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

    /// gets the best assignment for a ship, sets it directly in the database and returns the assignment id
    #[tracing::instrument(skip(self, ship_clone), name = "FleetManagerMessanger::get_assignment", fields(ship = %ship_clone.symbol))]
    pub async fn get_new_assignment(
        &self,
        ship_clone: &ship::MyShip,
    ) -> Result<Option<i64>, crate::error::Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.sender
            .send(FleetManagerMessage::GetNewAssignments {
                callback: sender,
                ship_clone: ship_clone.clone(),
                temp: false,
            })
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;
        let erg = receiver
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;
        Ok(erg)
    }

    /// gets the best temp assignment for a ship, sets it directly in the database and returns the assignment id
    #[tracing::instrument(skip(self, ship_clone), name = "FleetManagerMessanger::get_temp_assignment", fields(ship = %ship_clone.symbol))]
    pub async fn get_new_temp_assignment(
        &self,
        ship_clone: &ship::MyShip,
    ) -> Result<Option<i64>, crate::error::Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.sender
            .send(FleetManagerMessage::GetNewAssignments {
                callback: sender,
                ship_clone: ship_clone.clone(),
                temp: true,
            })
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;
        let erg = receiver
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;
        Ok(erg)
    }

    /// Ask the fleet manager to re-generate assignments for all fleets.
    pub async fn regenerate_all_assignments(&self) -> Result<(), crate::error::Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let erg = self
            .sender
            .send_timeout(
                FleetManagerMessage::ReGenerateAssignments { callback: sender },
                tokio::time::Duration::from_millis(5000),
            )
            .await;

        if let Err(e) = erg {
            match e {
                tokio::sync::mpsc::error::SendTimeoutError::Timeout(_e) => return Ok(()),
                tokio::sync::mpsc::error::SendTimeoutError::Closed(_e) => return Ok(()),
            }
        }

        receiver
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;

        Ok(())
    }

    /// Ask the fleet manager to re-generate assignments for all fleets.
    pub async fn regenerate_system_assignments(
        &self,
        system_symbol: String,
    ) -> Result<(), crate::error::Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let erg = self
            .sender
            .send_timeout(
                FleetManagerMessage::ReGenerateSystemAssignments {
                    callback: sender,
                    system_symbol,
                },
                tokio::time::Duration::from_millis(5000),
            )
            .await;

        if let Err(e) = erg {
            match e {
                tokio::sync::mpsc::error::SendTimeoutError::Timeout(_e) => return Ok(()),
                tokio::sync::mpsc::error::SendTimeoutError::Closed(_e) => return Ok(()),
            }
        }

        receiver
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;

        Ok(())
    }

    /// Ask the fleet manager to re-generate assignments for all fleets.
    pub async fn regenerate_fleet_assignments(
        &self,
        fleet_id: i32,
    ) -> Result<bool, crate::error::Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let erg = self
            .sender
            .send_timeout(
                FleetManagerMessage::ReGenerateFleetAssignments {
                    callback: sender,
                    fleet_id,
                },
                tokio::time::Duration::from_millis(5000),
            )
            .await;

        if let Err(e) = erg {
            match e {
                tokio::sync::mpsc::error::SendTimeoutError::Timeout(_e) => return Ok(false),
                tokio::sync::mpsc::error::SendTimeoutError::Closed(_e) => {
                    return Err(crate::error::Error::General("Channel Closed".to_string()))
                }
            }
        }

        receiver
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;

        Ok(false)
    }

    pub async fn populate_system(
        &self,
        system_symbol: String,
    ) -> Result<bool, crate::error::Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let erg = self.sender.try_send(FleetManagerMessage::PopulateSystem {
            callback: sender,
            system_symbol,
        });
        if let Err(e) = erg {
            match e {
                tokio::sync::mpsc::error::TrySendError::Full(_e) => return Ok(false),
                tokio::sync::mpsc::error::TrySendError::Closed(_e) => {
                    return Err(crate::error::Error::General("Channel Closed".to_string()))
                }
            }
        }
        receiver
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;
        Ok(true)
    }

    pub async fn populate_from_jump_gate(
        &self,
        jump_gate_symbol: String,
    ) -> Result<bool, crate::error::Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let erg = self
            .sender
            .try_send(FleetManagerMessage::PopulateFromJumpGate {
                callback: sender,
                jump_gate_symbol,
            });
        if let Err(e) = erg {
            match e {
                tokio::sync::mpsc::error::TrySendError::Full(_e) => return Ok(false),
                tokio::sync::mpsc::error::TrySendError::Closed(_e) => {
                    return Err(crate::error::Error::General("Channel Closed".to_string()))
                }
            }
        }
        receiver
            .await
            .map_err(|e| crate::error::Error::General(e.to_string()))?;
        Ok(true)
    }
}
