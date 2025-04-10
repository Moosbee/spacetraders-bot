use space_traders_client::models;

use crate::manager::fleet_manager::message::RequiredShips;

use crate::error::Result;

#[derive(Debug)]
pub enum ContractShipmentMessage {
    RequestNext {
        ship_clone: ship::MyShip,
        can_start_new_contract: bool,
        callback: tokio::sync::oneshot::Sender<Result<NextShipmentResp>>,
    },
    Failed {
        shipment: database::ContractShipment,
        error: crate::error::Error,
        callback: tokio::sync::oneshot::Sender<Result<crate::error::Error>>,
    },
    Finished {
        contract: models::Contract,
        shipment: database::ContractShipment,
    },
    GetRunning {
        callback: tokio::sync::oneshot::Sender<Result<Vec<database::ContractShipment>>>,
    },
    GetShips {
        callback: tokio::sync::oneshot::Sender<RequiredShips>,
    },
}

#[derive(Debug, Clone)]
pub enum NextShipmentResp {
    Shipment(database::ContractShipment),
    ComeBackLater,
}

pub type ContractManagerMessage = ContractShipmentMessage;
