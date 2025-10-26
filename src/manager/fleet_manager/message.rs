#[derive(Debug)]
pub enum FleetMessage {
    ScrapperAtShipyard {
        waypoint_symbol: String,
        ship_symbol: String,
        callback: tokio::sync::oneshot::Sender<String>,
    },
}

pub type FleetManagerMessage = FleetMessage;
