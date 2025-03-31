use crate::{error::Result, manager::fleet_manager::message::RequiredShips};

#[derive(Debug)]
pub enum MiningMessage {
    AssignWaypoint(AssignWaypointMessage),
    ExtractionNotification(ExtractionNotification),
    GetPlaces {
        callback:
            tokio::sync::oneshot::Sender<Result<Vec<(String, super::mining_places::WaypointInfo)>>>,
    },
    GetShips {
        callback: tokio::sync::oneshot::Sender<RequiredShips>,
    },
}

impl std::fmt::Display for MiningMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssignWaypoint(inner) => write!(f, "AssignWaypoint: {}", inner),
            Self::ExtractionNotification(inner) => write!(f, "ExtractionNotification: {}", inner),
            Self::GetPlaces { callback } => write!(f, "GetPlaces: {:?}", callback),
            Self::GetShips { callback } => write!(f, "GetShips: {:?}", callback),
        }
    }
}

pub type MiningManagerMessage = MiningMessage;

#[derive(Debug)]
pub enum AssignWaypointMessage {
    AssignWaypoint {
        // assigns a ship to a waypoint, ship might need to get there
        ship_clone: crate::ship::MyShip,
        is_syphon: bool,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
    NotifyWaypoint {
        // assigns a ship to a waypoint(level two), ship is now there
        ship_clone: crate::ship::MyShip,
        is_syphon: bool,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
    UnassignWaypoint {
        // unassigns a ship from level two to level one
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
    #[allow(dead_code)]
    UnassignWaypointComplete {
        // unassigns a ship from level two to level one
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
}

impl std::fmt::Display for AssignWaypointMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignWaypointMessage::AssignWaypoint {
                ship_clone,
                is_syphon,
                callback: _,
            } => write!(
                f,
                "AssignWaypoint for {} is_syphon {}",
                ship_clone.symbol, is_syphon
            ),
            AssignWaypointMessage::NotifyWaypoint {
                ship_clone,
                is_syphon,
                callback: _,
            } => write!(
                f,
                "NotifyWaypoint for {} is_syphon {}",
                ship_clone.symbol, is_syphon
            ),
            AssignWaypointMessage::UnassignWaypoint {
                ship_clone,
                callback: _,
            } => write!(f, "UnassignWaypoint for {}", ship_clone.symbol),
            AssignWaypointMessage::UnassignWaypointComplete {
                ship_clone,
                callback: _,
            } => write!(f, "UnassignWaypointComplete for {}", ship_clone.symbol),
        }
    }
}

#[derive(Debug)]
pub enum ExtractionNotification {
    GetNextWaypoint {
        // when a transporter is empty and wants to find a new waypoint, acording to it's urgency
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<String>>,
    },
    ExtractionComplete {
        // when a ship completed an extraction
        #[allow(dead_code)]
        ship: String,
        waypoint: String,
    },
    TransportArrived {
        // when a transporter ship arrived
        #[allow(dead_code)]
        ship: String,
        waypoint: String,
    },
    // ExtractorContact {
    //     symbol: String,
    //     sender: tokio::sync::mpsc::Sender<ExtractorTransferRequest>,
    // },
    // TransportationContact {
    //     symbol: String,
    //     sender: tokio::sync::mpsc::Sender<TransportTransferRequest>,
    // },
}

impl std::fmt::Display for ExtractionNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionNotification::GetNextWaypoint {
                ship_clone,
                callback: _,
            } => write!(f, "GetNextWaypoint for {}", ship_clone.symbol),
            ExtractionNotification::ExtractionComplete { ship, waypoint } => write!(
                f,
                "ExtractionComplete for {} at waypoint {}",
                ship, waypoint
            ),
            ExtractionNotification::TransportArrived { ship, waypoint } => {
                write!(f, "TransportArrived for {} at waypoint {}", ship, waypoint)
            }
        }
    }
}
