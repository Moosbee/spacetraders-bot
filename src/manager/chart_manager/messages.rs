use space_traders_client::WaypointSymbol;

#[derive(Debug)]
pub enum ChartMessage {
    Next {
        ship_clone: ship::MyShipCopy,
        chart_only_jump_gates: bool,
        callback: tokio::sync::oneshot::Sender<Result<NextChartResp, crate::error::Error>>,
    },
    Fail {
        waypoint_symbol: WaypointSymbol,
    },
    Success {
        waypoint_symbol: WaypointSymbol,
    },
}

#[derive(Debug, Clone)]
pub enum NextChartResp {
    Next(WaypointSymbol),
    NoChartsInSystem,
}

pub type ChartManagerMessage = ChartMessage;
