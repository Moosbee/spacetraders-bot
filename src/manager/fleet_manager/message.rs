#[derive(Debug)]
pub enum FleetMessage {
    ScrapperAtShipyard {
        waypoint_symbol: String,
        ship_symbol: String,
        callback: tokio::sync::oneshot::Sender<String>,
    },
    GetNewAssignments {
        callback: tokio::sync::oneshot::Sender<Option<i64>>,
        ship_clone: ship::MyShip,
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
        system_symbol: String,
    },
    PopulateSystem {
        callback: tokio::sync::oneshot::Sender<()>,
        system_symbol: String,
    },
    PopulateFromJumpGate {
        callback: tokio::sync::oneshot::Sender<()>,
        jump_gate_symbol: String,
    },
}

pub type FleetManagerMessage = FleetMessage;
