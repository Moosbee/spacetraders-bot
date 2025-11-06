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
}

pub type FleetManagerMessage = FleetMessage;
