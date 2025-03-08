#[derive(Debug, Default, Clone, serde::Serialize, PartialEq, Eq)]
pub enum MiningShipAssignment {
    Transporter {
        state: TransporterState,
        waypoint_symbol: String,
        cycles: i32,
    },
    Extractor {
        state: ExtractorState,
        waypoint_symbol: String,
        extractions: i32,
    },
    Siphoner {
        state: SiphonerState,
        waypoint_symbol: String,
        extractions: i32,
    },
    Surveyor,
    #[default]
    Idle,
    Useless,
}

pub enum TransporterState {
    InTransitToAsteroid,
    LoadingCargo,
    WaitingForCargo,
    InTransitToMarket,
    SellingCargo,
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
