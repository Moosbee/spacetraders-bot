use space_traders_client::models;

use crate::{ship, sql};

#[derive(Debug)]
pub enum ConstructionMessage {
    RequestNextShipment {
        ship_clone: ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<NextShipmentResp, crate::error::Error>>,
    },
    FailedShipment {
        shipment: sql::ConstructionShipment,
        error: crate::error::Error,
        callback: tokio::sync::oneshot::Sender<crate::error::Error>,
    },
    FinishedShipment {
        construction: models::Construction,
        shipment: sql::ConstructionShipment,
    },
    GetRunning {
        callback: tokio::sync::oneshot::Sender<
            Result<Vec<sql::ConstructionShipment>, crate::error::Error>,
        >,
    },
}

#[derive(Debug, Clone)]
pub enum NextShipmentResp {
    Shipment(sql::ConstructionShipment),
    ComeBackLater,
}

pub type ConstructionManagerMessage = ConstructionMessage;
