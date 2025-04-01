use space_traders_client::models;

use crate::manager::fleet_manager::message::RequiredShips;
use crate::{ship, sql};

use crate::error::Result;

#[derive(Debug)]
pub enum ContractShipmentMessage {
    RequestNext {
        ship_clone: ship::MyShip,
        can_start_new_contract: bool,
        callback: tokio::sync::oneshot::Sender<Result<NextShipmentResp>>,
    },
    Failed {
        shipment: sql::ContractShipment,
        error: crate::error::Error,
        callback: tokio::sync::oneshot::Sender<Result<crate::error::Error>>,
    },
    Finished {
        contract: models::Contract,
        shipment: sql::ContractShipment,
    },
    GetRunning {
        callback: tokio::sync::oneshot::Sender<Result<Vec<sql::ContractShipment>>>,
    },
    GetShips {
        callback: tokio::sync::oneshot::Sender<RequiredShips>,
    },
}

#[derive(Debug, Clone)]
pub enum NextShipmentResp {
    Shipment(sql::ContractShipment),
    ComeBackLater,
}

pub type ContractManagerMessage = ContractShipmentMessage;
