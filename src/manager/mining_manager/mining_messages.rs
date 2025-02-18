use crate::error::Result;

use super::transfer_manager::{ExtractorTransferRequest, TransportTransferRequest};

#[derive(Debug)]
pub enum MiningMessage {
    AssignWaypoint(AssignWaypointMessage),
    ExtractionNotification(ExtractionNotification),
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
    ExtractorContact {
        symbol: String,
        sender: tokio::sync::mpsc::Sender<ExtractorTransferRequest>,
    },
    TransportationContact {
        symbol: String,
        sender: tokio::sync::mpsc::Sender<TransportTransferRequest>,
    },
}
