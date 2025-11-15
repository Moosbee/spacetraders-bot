use async_graphql::{Enum, Union};

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
#[graphql(complex)]
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

#[async_graphql::ComplexObject]
impl ShipStatus {
    async fn status(&self) -> AssignmentStatusGQL {
        self.status.clone().into()
    }

    async fn assignment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::ShipAssignment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(assignment) = self.assignment_id {
            let erg = database::ShipAssignment::get_by_id(database_pool, assignment).await?;
            Ok(erg)
        } else {
            Ok(None)
        }
    }

    async fn temp_assignment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::ShipAssignment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(assignment) = self.temp_assignment_id {
            let erg = database::ShipAssignment::get_by_id(database_pool, assignment).await?;
            Ok(erg)
        } else {
            Ok(None)
        }
    }

    async fn fleet<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::Fleet>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(fleet) = self.fleet_id {
            let erg = database::Fleet::get_by_id(database_pool, fleet).await?;
            Ok(erg)
        } else {
            Ok(None)
        }
    }

    async fn temp_fleet<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> crate::Result<Option<database::Fleet>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(fleet) = self.temp_fleet_id {
            let erg = database::Fleet::get_by_id(database_pool, fleet).await?;
            Ok(erg)
        } else {
            Ok(None)
        }
    }
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

// Mining assignment variants
#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct TransporterAssignment {
    pub state: TransporterState,
    pub waypoint_symbol: Option<String>,
    pub cycles: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct ExtractorAssignment {
    pub state: ExtractorState,
    pub waypoint_symbol: Option<String>,
    pub extractions: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct SiphonerAssignment {
    pub state: ExtractorState,
    pub waypoint_symbol: Option<String>,
    pub extractions: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct SurveyorAssignment {
    pub waypoint_symbol: Option<String>,
    pub surveys: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct IdleAssignment {
    pub controlled: bool,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct UselessAssignment {
    pub controlled: bool,
}

#[derive(Debug, Clone, serde::Serialize, Union)]
#[graphql(name = "MiningShipAssignment")]
pub enum MiningShipAssignmentGQL {
    Transporter(TransporterAssignment),
    Extractor(ExtractorAssignment),
    Siphoner(SiphonerAssignment),
    Surveyor(SurveyorAssignment),
    Idle(IdleAssignment),
    Useless(UselessAssignment),
}

// Assignment status variants
#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct ConstructionStatus {
    pub cycle: Option<i32>,
    pub shipment_id: Option<i64>,
    pub shipping_status: Option<ShippingStatus>,
    pub waiting_for_manager: bool,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct TraderStatus {
    pub shipment_id: Option<i32>,
    pub cycle: Option<i32>,
    pub shipping_status: Option<ShippingStatus>,
    pub waiting_for_manager: bool,
    pub on_sleep: bool,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct TransferStatus {
    pub fleet_id: i32,
    pub assignment_id: i64,
    pub system_symbol: String,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct ContractStatus {
    pub contract_id: Option<String>,
    pub run_id: Option<i32>,
    pub cycle: Option<i32>,
    pub shipping_status: Option<ShippingStatus>,
    pub waiting_for_manager: bool,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct ScraperStatus {
    pub cycle: Option<i32>,
    pub waiting_for_manager: bool,
    pub waypoint_symbol: Option<String>,
    pub scrap_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct MiningStatus {
    pub assignment: MiningShipAssignmentGQL,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct ChartingStatus {
    pub cycle: Option<i32>,
    pub waiting_for_manager: bool,
    pub waypoint_symbol: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct ManuelStatus {
    pub controlled: bool,
}

#[derive(Debug, Clone, Union)]
#[graphql(name = "AssignmentStatus")]
pub enum AssignmentStatusGQL {
    Construction(ConstructionStatus),
    Trader(TraderStatus),
    Transfer(TransferStatus),
    Contract(ContractStatus),
    Scraper(ScraperStatus),
    Mining(MiningStatus),
    Charting(ChartingStatus),
    Manuel(ManuelStatus),
}

// Conversions
impl From<MiningShipAssignment> for MiningShipAssignmentGQL {
    fn from(assignment: MiningShipAssignment) -> Self {
        match assignment {
            MiningShipAssignment::Transporter {
                state,
                waypoint_symbol,
                cycles,
            } => MiningShipAssignmentGQL::Transporter(TransporterAssignment {
                state,
                waypoint_symbol,
                cycles,
            }),
            MiningShipAssignment::Extractor {
                state,
                waypoint_symbol,
                extractions,
            } => MiningShipAssignmentGQL::Extractor(ExtractorAssignment {
                state,
                waypoint_symbol,
                extractions,
            }),
            MiningShipAssignment::Siphoner {
                state,
                waypoint_symbol,
                extractions,
            } => MiningShipAssignmentGQL::Siphoner(SiphonerAssignment {
                state,
                waypoint_symbol,
                extractions,
            }),
            MiningShipAssignment::Surveyor {
                waypoint_symbol,
                surveys,
            } => MiningShipAssignmentGQL::Surveyor(SurveyorAssignment {
                waypoint_symbol,
                surveys,
            }),
            MiningShipAssignment::Idle => {
                MiningShipAssignmentGQL::Idle(IdleAssignment { controlled: false })
            }
            MiningShipAssignment::Useless => {
                MiningShipAssignmentGQL::Useless(UselessAssignment { controlled: false })
            }
        }
    }
}

impl From<AssignmentStatus> for AssignmentStatusGQL {
    fn from(status: AssignmentStatus) -> Self {
        match status {
            AssignmentStatus::Construction {
                cycle,
                shipment_id,
                shipping_status,
                waiting_for_manager,
            } => AssignmentStatusGQL::Construction(ConstructionStatus {
                cycle,
                shipment_id,
                shipping_status,
                waiting_for_manager,
            }),
            AssignmentStatus::Trader {
                shipment_id,
                cycle,
                shipping_status,
                waiting_for_manager,
                on_sleep,
            } => AssignmentStatusGQL::Trader(TraderStatus {
                shipment_id,
                cycle,
                shipping_status,
                waiting_for_manager,
                on_sleep,
            }),
            AssignmentStatus::Transfer {
                fleet_id,
                assignment_id,
                system_symbol,
            } => AssignmentStatusGQL::Transfer(TransferStatus {
                fleet_id,
                assignment_id,
                system_symbol,
            }),
            AssignmentStatus::Contract {
                contract_id,
                run_id,
                cycle,
                shipping_status,
                waiting_for_manager,
            } => AssignmentStatusGQL::Contract(ContractStatus {
                contract_id,
                run_id,
                cycle,
                shipping_status,
                waiting_for_manager,
            }),
            AssignmentStatus::Scraper {
                cycle,
                waiting_for_manager,
                waypoint_symbol,
                scrap_date,
            } => AssignmentStatusGQL::Scraper(ScraperStatus {
                cycle,
                waiting_for_manager,
                waypoint_symbol,
                scrap_date,
            }),
            AssignmentStatus::Mining { assignment } => AssignmentStatusGQL::Mining(MiningStatus {
                assignment: assignment.into(),
            }),
            AssignmentStatus::Charting {
                cycle,
                waiting_for_manager,
                waypoint_symbol,
            } => AssignmentStatusGQL::Charting(ChartingStatus {
                cycle,
                waiting_for_manager,
                waypoint_symbol,
            }),
            AssignmentStatus::Manuel => {
                AssignmentStatusGQL::Manuel(ManuelStatus { controlled: false })
            }
        }
    }
}
