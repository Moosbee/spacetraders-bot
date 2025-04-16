#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub enum ShippingStatus {
    InTransitToPurchase,
    Purchasing,
    InTransitToDelivery,
    Delivering,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ShipStatus {
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
    Transfer {
        id: Option<i64>,
        system_symbol: Option<String>,
        role: Option<database::ShipInfoRole>,
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
    Surveyor,
    #[default]
    Idle,
    Useless,
}

#[derive(Debug, Default, Clone, Copy, serde::Serialize, PartialEq, Eq)]
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

#[derive(Debug, Default, Clone, Copy, serde::Serialize, PartialEq, Eq)]
pub enum ExtractorState {
    InTransit,
    Mining,
    OnCooldown,
    InvFull,
    #[default]
    Unknown,
}
