use async_graphql::Enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, Enum)]
pub enum ShippingStatus {
    InTransitToPurchase,
    Purchasing,
    InTransitToDelivery,
    Delivering,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "ShipShipStatus")]
pub struct ShipStatus {
    pub waiting_for_manager: bool,
    pub waiting_for_api: bool,
    pub fleet_id: Option<i32>,
    pub temp_fleet_id: Option<i32>,
    pub assignment_id: Option<i64>,
    pub temp_assignment_id: Option<i64>,
    #[graphql(skip)]
    pub status: AssignmentStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum AssignmentStatus {
    Construction {
        cycle: Option<i32>,
        shipment_id: Option<i64>,
        shipping_status: Option<ShippingStatus>,
        waiting_for_manager: bool,
    },
    Trader {
        shipment_id: Option<i32>,
        cycle: Option<i32>,
        shipping_status: Option<ShippingStatus>,
        waiting_for_manager: bool,
        on_sleep: bool,
    },
    Transfer {
        fleet_id: i32,
        assignment_id: i64,
        system_symbol: String,
    },
    Contract {
        contract_id: Option<String>,
        run_id: Option<i32>,
        cycle: Option<i32>,
        shipping_status: Option<ShippingStatus>,
        waiting_for_manager: bool,
    },
    Scraper {
        cycle: Option<i32>,
        waiting_for_manager: bool,
        waypoint_symbol: Option<String>,
        scrap_date: Option<chrono::DateTime<chrono::Utc>>,
    },
    Mining {
        assignment: MiningShipAssignment,
    },
    Charting {
        cycle: Option<i32>,
        waiting_for_manager: bool,
        waypoint_symbol: Option<String>,
    },
    #[default]
    Manuel,
}

#[derive(Debug, Default, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum MiningShipAssignment {
    Transporter {
        state: TransporterState,
        waypoint_symbol: Option<String>,
        cycles: Option<i32>,
    },
    Extractor {
        state: ExtractorState,
        waypoint_symbol: Option<String>,
        extractions: Option<i32>,
    },
    Siphoner {
        state: SiphonerState,
        waypoint_symbol: Option<String>,
        extractions: Option<i32>,
    },
    Surveyor {
        waypoint_symbol: Option<String>,
        surveys: Option<i32>,
    },
    #[default]
    Idle,
    Useless,
}

#[derive(Debug, Default, Clone, Copy, serde::Serialize, PartialEq, Eq, Enum)]
pub enum TransporterState {
    InTransitToAsteroid,
    LoadingCargo,
    WaitingForCargo,
    InTransitToMarket,
    SellingCargo,
    #[default]
    Unknown,
}

pub type SiphonerState = ExtractorState;

#[derive(Debug, Default, Clone, Copy, serde::Serialize, PartialEq, Eq, Enum)]
pub enum ExtractorState {
    InTransit,
    Mining,
    OnCooldown,
    InvFull,
    #[default]
    Unknown,
}
