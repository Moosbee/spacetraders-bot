use crate::manager::fleet_manager::message::RequiredShips;

#[derive(Debug)]
pub enum ChartMessage {
    Next {
        ship_clone: crate::ship::MyShip,
        callback: tokio::sync::oneshot::Sender<Result<NextChartResp, crate::error::Error>>,
    },
    Fail {
        waypoint_symbol: String,
    },
    Success {
        waypoint_symbol: String,
    },
    GetShips {
        callback: tokio::sync::oneshot::Sender<RequiredShips>,
    },
}

#[derive(Debug, Clone)]
pub enum NextChartResp {
    Next(String),
    NoChartsInSystem,
}

pub type ChartManagerMessage = ChartMessage;
