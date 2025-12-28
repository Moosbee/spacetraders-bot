use std::{collections::HashMap, sync::Arc};

use async_graphql::{dataloader::DataLoader, Union};
use ship::{
    status::{ExtractorState, MiningShipAssignment, ShipStatus, TransporterState},
    AssignmentStatus, AutopilotState, ModuleState, MountState, NavigationState, RouteState,
    ShippingStatus,
};
use space_traders_client::models;
use tracing::instrument;
use utils::get_system_symbol;

use crate::{control_api::graphql::gql_models, error::Result};

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipStatus")]
#[graphql(complex)]
pub struct GQLShipStatus {
    #[graphql(flatten)]
    ship_status: ShipStatus,
}

impl From<ShipStatus> for GQLShipStatus {
    fn from(value: ShipStatus) -> Self {
        GQLShipStatus { ship_status: value }
    }
}

#[async_graphql::ComplexObject]
impl GQLShipStatus {
    async fn status(&self) -> AssignmentStatusGQL {
        self.ship_status.status.clone().into()
    }

    async fn assignment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLShipAssignment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(assignment) = self.ship_status.assignment_id {
            let erg = database::ShipAssignment::get_by_id(database_pool, assignment).await?;
            Ok(erg.map(|f| f.into()))
        } else {
            Ok(None)
        }
    }

    async fn temp_assignment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLShipAssignment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(assignment) = self.ship_status.temp_assignment_id {
            let erg = database::ShipAssignment::get_by_id(database_pool, assignment).await?;
            Ok(erg.map(|f| f.into()))
        } else {
            Ok(None)
        }
    }

    async fn fleet<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLFleet>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(fleet) = self.ship_status.fleet_id {
            let erg = database::Fleet::get_by_id(database_pool, fleet).await?;
            Ok(erg.map(|f| f.into()))
        } else {
            Ok(None)
        }
    }

    async fn temp_fleet<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLFleet>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(fleet) = self.ship_status.temp_fleet_id {
            let erg = database::Fleet::get_by_id(database_pool, fleet).await?;
            Ok(erg.map(|f| f.into()))
        } else {
            Ok(None)
        }
    }
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
#[graphql(complex)]
pub struct ConstructionStatus {
    pub cycle: Option<i32>,
    pub shipment_id: Option<i64>,
    pub shipping_status: Option<ShippingStatus>,
    pub waiting_for_manager: bool,
}

#[async_graphql::ComplexObject]
impl ConstructionStatus {
    async fn shipment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLConstructionShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(shipment_id) = self.shipment_id {
            let erg = database::ConstructionShipment::get_by_id(database_pool, shipment_id).await?;
            Ok(erg.map(|f| f.into()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct TraderStatus {
    pub shipment_id: Option<i32>,
    pub cycle: Option<i32>,
    pub shipping_status: Option<ShippingStatus>,
    pub waiting_for_manager: bool,
    pub on_sleep: bool,
}

#[async_graphql::ComplexObject]
impl TraderStatus {
    async fn trade_route<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLTradeRoute>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(shipment_id) = self.shipment_id {
            let erg = database::TradeRoute::get_by_id(database_pool, shipment_id).await?;
            Ok(erg.map(|f| f.into()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct TransferStatus {
    pub fleet_id: i32,
    pub assignment_id: i64,
    pub system_symbol: String,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct ContractStatus {
    pub contract_id: Option<String>,
    pub run_id: Option<i32>,
    pub cycle: Option<i32>,
    pub shipping_status: Option<ShippingStatus>,
    pub waiting_for_manager: bool,
}

#[async_graphql::ComplexObject]
impl ContractStatus {
    async fn contract<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLContract>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(contract_id) = &self.contract_id {
            let erg = database::Contract::get_by_id(database_pool, contract_id).await?;
            Ok(erg.map(|f| f.into()))
        } else {
            Ok(None)
        }
    }

    async fn contract_run<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLContractShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(run_id) = &self.run_id {
            let erg = database::ContractShipment::get_by_id(database_pool, *run_id).await?;
            Ok(Some(erg).map(|f| f.into()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct ScraperStatus {
    pub cycle: Option<i32>,
    pub waiting_for_manager: bool,
    pub waypoint_symbol: Option<String>,
    pub scrap_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_graphql::ComplexObject]
impl ScraperStatus {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        if let Some(waypoint_symbol) = &self.waypoint_symbol {
            let erg = data_loader.load_one(waypoint_symbol.to_string()).await?;
            Ok(erg.map(|f| f.into()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct MiningStatus {
    pub assignment: MiningShipAssignmentGQL,
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct ChartingStatus {
    pub cycle: Option<i32>,
    pub waiting_for_manager: bool,
    pub waypoint_symbol: Option<String>,
}

#[async_graphql::ComplexObject]
impl ChartingStatus {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        if let Some(waypoint_symbol) = &self.waypoint_symbol {
            let erg = data_loader.load_one(waypoint_symbol.clone()).await?;
            Ok(erg.map(|f| f.into()))
        } else {
            Ok(None)
        }
    }
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Mounts")]
#[graphql(complex)]
pub struct GQLMounts {
    #[graphql(flatten)]
    mounts: MountState,
}

impl From<MountState> for GQLMounts {
    fn from(value: MountState) -> Self {
        GQLMounts { mounts: value }
    }
}

#[async_graphql::ComplexObject]
impl GQLMounts {
    async fn mount_infos<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLMountInfo>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let mut mounts = Vec::new();
        for mount_symbol in self.mounts.mounts.iter() {
            let erg = database::MountInfo::get_by_id(database_pool, mount_symbol).await?;
            mounts.push(erg);
        }
        Ok(mounts.into_iter().map(|m| m.into()).collect())
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Modules")]
#[graphql(complex)]
pub struct GQLModules {
    #[graphql(flatten)]
    modules: ModuleState,
}

impl From<ModuleState> for GQLModules {
    fn from(value: ModuleState) -> Self {
        GQLModules { modules: value }
    }
}

#[async_graphql::ComplexObject]
impl GQLModules {
    async fn module_infos<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLModuleInfo>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let mut modules = Vec::new();
        for module_symbol in self.modules.modules.iter() {
            let erg = database::ModuleInfo::get_by_id(database_pool, module_symbol).await?;
            modules.push(erg);
        }
        Ok(modules.into_iter().map(|m| m.into()).collect())
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "NavigationState")]
#[graphql(complex)]
pub struct GQLNavigationState {
    #[graphql(flatten)]
    nav: NavigationState,
}

impl From<NavigationState> for GQLNavigationState {
    fn from(value: NavigationState) -> Self {
        GQLNavigationState { nav: value }
    }
}

#[async_graphql::ComplexObject]
impl GQLNavigationState {
    async fn system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::System::get_by_symbol(database_pool, &self.nav.system_symbol).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.nav.waypoint_symbol.clone())
            .await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn route(&self) -> Result<GQLRouteState> {
        Ok(self.nav.route.clone().into())
    }

    async fn auto_pilot(&self) -> Result<Option<GQLAutopilotState>> {
        Ok(self.nav.auto_pilot.clone().map(|f| f.into()))
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "RouteState")]
#[graphql(complex)]
pub struct GQLRouteState {
    #[graphql(flatten)]
    route: RouteState,
}

impl From<RouteState> for GQLRouteState {
    fn from(value: RouteState) -> Self {
        GQLRouteState { route: value }
    }
}

#[async_graphql::ComplexObject]
impl GQLRouteState {
    async fn destination_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::System::get_by_symbol(database_pool, &self.route.destination_system_symbol)
                .await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn destination_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.route.destination_symbol.clone())
            .await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn origin_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::System::get_by_symbol(database_pool, &self.route.origin_system_symbol)
            .await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn origin_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.route.origin_symbol.clone())
            .await?;
        Ok(erg.map(|f| f.into()))
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "AutopilotState")]
#[graphql(complex)]
pub struct GQLAutopilotState {
    #[graphql(flatten)]
    auto_pilot: AutopilotState,
}

impl From<AutopilotState> for GQLAutopilotState {
    fn from(value: AutopilotState) -> Self {
        GQLAutopilotState { auto_pilot: value }
    }
}

#[async_graphql::ComplexObject]
impl GQLAutopilotState {
    async fn route(&self) -> RouteGQL {
        self.auto_pilot.route.clone().into()
    }

    async fn destination_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::System::get_by_symbol(
            database_pool,
            &self.auto_pilot.destination_system_symbol,
        )
        .await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn destination_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.auto_pilot.destination_symbol.clone())
            .await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn origin_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::System::get_by_symbol(database_pool, &self.auto_pilot.origin_system_symbol)
                .await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn origin_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.auto_pilot.origin_symbol.clone())
            .await?;
        Ok(erg.map(|f| f.into()))
    }
}

// Connection variants as GraphQL objects
#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "JumpConnection")]
#[graphql(complex)]
pub struct JumpConnectionGQL {
    pub start_symbol: String,
    pub end_symbol: String,
    pub distance: f64,
    pub cooldown_time: f64,
}

#[async_graphql::ComplexObject]
impl JumpConnectionGQL {
    async fn start<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.start_symbol.clone()).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn start_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.start_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn end<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.end_symbol.clone()).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn end_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.end_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg.map(|f| f.into()))
    }
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "WarpConnection")]
#[graphql(complex)]
pub struct WarpConnectionGQL {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: ship::autopilot::Refuel,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[async_graphql::ComplexObject]
impl WarpConnectionGQL {
    async fn start<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.start_symbol.clone()).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn start_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.start_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn end<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.end_symbol.clone()).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn end_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.end_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg.map(|f| f.into()))
    }
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "NavigateConnection")]
#[graphql(complex)]
pub struct NavigateConnectionGQL {
    pub start_symbol: String,
    pub end_symbol: String,
    pub nav_mode: models::ShipNavFlightMode,
    pub distance: f64,
    pub travel_time: f64,
    pub refuel: ship::autopilot::Refuel,
    pub start_is_marketplace: bool,
    pub end_is_marketplace: bool,
}

#[async_graphql::ComplexObject]
impl NavigateConnectionGQL {
    async fn start<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.start_symbol.clone()).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn start_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.start_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn end<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.end_symbol.clone()).await?;
        Ok(erg.map(|f| f.into()))
    }

    async fn end_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<gql_models::GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = get_system_symbol(&self.end_symbol);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg.map(|f| f.into()))
    }
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::Union)]
#[graphql(name = "ConcreteConnection")]
pub enum ConcreteConnectionGQL {
    JumpGate(JumpConnectionGQL),
    Warp(WarpConnectionGQL),
    Navigate(NavigateConnectionGQL),
}

#[derive(Clone, Default, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(name = "AutopilotRoute")]
pub struct RouteGQL {
    pub connections: Vec<ConcreteConnectionGQL>,
    pub total_distance: f64,
    pub total_fuel_cost: f64,
    pub total_travel_time: f64,
    pub total_api_requests: i32,
}

impl From<ship::autopilot::JumpConnection> for JumpConnectionGQL {
    fn from(conn: ship::autopilot::JumpConnection) -> Self {
        JumpConnectionGQL {
            start_symbol: conn.start_symbol,
            end_symbol: conn.end_symbol,
            distance: conn.distance,
            cooldown_time: conn.cooldown_time,
        }
    }
}

impl From<ship::autopilot::WarpConnection> for WarpConnectionGQL {
    fn from(conn: ship::autopilot::WarpConnection) -> Self {
        WarpConnectionGQL {
            start_symbol: conn.start_symbol,
            end_symbol: conn.end_symbol,
            nav_mode: conn.nav_mode,
            distance: conn.distance,
            travel_time: conn.travel_time,
            refuel: conn.refuel,
            start_is_marketplace: conn.start_is_marketplace,
            end_is_marketplace: conn.end_is_marketplace,
        }
    }
}

impl From<ship::autopilot::NavigateConnection> for NavigateConnectionGQL {
    fn from(conn: ship::autopilot::NavigateConnection) -> Self {
        NavigateConnectionGQL {
            start_symbol: conn.start_symbol,
            end_symbol: conn.end_symbol,
            nav_mode: conn.nav_mode,
            distance: conn.distance,
            travel_time: conn.travel_time,
            refuel: conn.refuel,
            start_is_marketplace: conn.start_is_marketplace,
            end_is_marketplace: conn.end_is_marketplace,
        }
    }
}

impl From<ship::autopilot::ConcreteConnection> for ConcreteConnectionGQL {
    fn from(conn: ship::autopilot::ConcreteConnection) -> Self {
        match conn {
            ship::autopilot::ConcreteConnection::JumpGate(jump) => {
                ConcreteConnectionGQL::JumpGate(jump.into())
            }
            ship::autopilot::ConcreteConnection::Warp(warp) => {
                ConcreteConnectionGQL::Warp(warp.into())
            }
            ship::autopilot::ConcreteConnection::Navigate(nav) => {
                ConcreteConnectionGQL::Navigate(nav.into())
            }
        }
    }
}

impl From<ship::autopilot::Route> for RouteGQL {
    fn from(route: ship::autopilot::Route) -> Self {
        RouteGQL {
            connections: route.connections.into_iter().map(|c| c.into()).collect(),
            total_distance: route.total_distance,
            total_fuel_cost: route.total_fuel_cost,
            total_travel_time: route.total_travel_time,
            total_api_requests: route.total_api_requests,
        }
    }
}

pub struct AllShipLoader(crate::utils::ConductorContext);

impl AllShipLoader {
    pub fn new(context: crate::utils::ConductorContext) -> Self {
        Self(context)
    }
}

impl async_graphql::dataloader::Loader<()> for AllShipLoader {
    type Value = HashMap<String, ship::RustShip<ShipStatus>>;
    type Error = Arc<crate::error::Error>;

    #[instrument(level = "trace", skip(self, keys))]
    async fn load(
        &self,
        keys: &[()],
    ) -> std::result::Result<HashMap<(), Self::Value>, Self::Error> {
        // let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let mut map = HashMap::new();
        let all_ships = self.0.ship_manager.get_all_clone().await;
        map.insert((), all_ships);
        Ok(map)
    }
}
