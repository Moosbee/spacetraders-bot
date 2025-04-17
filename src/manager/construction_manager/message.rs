use space_traders_client::models;

#[derive(Debug)]
pub enum ConstructionMessage {
    RequestNextShipment {
        ship_clone: ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<NextShipmentResp, crate::error::Error>>,
    },
    FailedShipment {
        shipment: database::ConstructionShipment,
        error: crate::error::Error,
        callback: tokio::sync::oneshot::Sender<crate::error::Error>,
    },
    FinishedShipment {
        construction: models::Construction,
        shipment: database::ConstructionShipment,
    },
    GetRunning {
        callback: tokio::sync::oneshot::Sender<
            Result<Vec<database::ConstructionShipment>, crate::error::Error>,
        >,
    },
}

#[derive(Debug, Clone)]
pub enum NextShipmentResp {
    Shipment(database::ConstructionShipment),
    ComeBackLater,
}

pub type ConstructionManagerMessage = ConstructionMessage;
