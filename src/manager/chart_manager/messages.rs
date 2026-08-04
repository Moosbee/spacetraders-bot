#[derive(Debug)]
pub enum ChartMessage {
    Next {
        ship_clone: ship::MyShipCopy,
        chart_only_jump_gates: bool,
        callback: tokio::sync::oneshot::Sender<Result<NextChartResp, crate::error::Error>>,
    },
    Fail {
        waypoint_symbol: String,
    },
    Success {
        waypoint_symbol: String,
    },
}

#[derive(Debug, Clone)]
pub enum NextChartResp {
    Next(String),
    NoChartsInSystem,
}

pub type ChartManagerMessage = ChartMessage;
