use space_traders_client::{ShipSymbol, SystemSymbol, WaypointSymbol};

#[derive(Debug)]
pub enum FleetMessage {
    GetNewAssignments {
        callback: tokio::sync::oneshot::Sender<Option<i64>>,
        ship_clone: ship::MyShipCopy,
        temp: bool,
    },
    ReGenerateAssignments {
        callback: tokio::sync::oneshot::Sender<()>,
    },
    ReGenerateFleetAssignments {
        callback: tokio::sync::oneshot::Sender<()>,
        fleet_id: i32,
    },
    ReGenerateSystemAssignments {
        callback: tokio::sync::oneshot::Sender<()>,
        system_symbol: SystemSymbol,
    },
    PopulateSystem {
        callback: tokio::sync::oneshot::Sender<()>,
        system_symbol: SystemSymbol,
    },
    PopulateFromJumpGate {
        callback: tokio::sync::oneshot::Sender<()>,
        jump_gate_symbol: WaypointSymbol,
    },
}

pub type FleetManagerMessage = FleetMessage;

#[derive(Debug)]
pub enum ShipProcurementMessage {
    ScrapperAtShipyard {
        waypoint_symbol: WaypointSymbol,
        ship_symbol: ShipSymbol,
        callback: tokio::sync::oneshot::Sender<ShipSymbol>,
    },
}
