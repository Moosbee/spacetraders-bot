use std::collections::HashMap;

use space_traders_client::models;

use crate::error::Result;

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Agent")]
#[graphql(complex)]
pub struct GQLAgent {
    #[graphql(flatten)]
    agent: database::Agent,
}

impl From<database::Agent> for GQLAgent {
    fn from(value: database::Agent) -> Self {
        GQLAgent { agent: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLAgent {
    async fn history<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::Agent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::Agent::get_by_symbol(database_pool, &self.symbol).await?;
        Ok(reg)
    }

    async fn headquarters_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::Waypoint::get_by_symbol(database_pool, &self.headquarters).await?;
        Ok(reg)
    }

    async fn headquarters_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::System>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = utils::get_system_symbol(&self.headquarters);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(erg)
    }
}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ChartTransaction")]
#[graphql(complex)]
pub struct GQLChartTransaction {
    #[graphql(flatten)]
    chart_transaction: database::ChartTransaction,
}

impl From<database::ChartTransaction> for GQLChartTransaction {
    fn from(value: database::ChartTransaction) -> Self {
        GQLChartTransaction {
            chart_transaction: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLChartTransaction {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await?;
        Ok(waypoint)
    }
}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ConstructionMaterial")]
#[graphql(complex)]
pub struct GQLConstructionMaterial {
    #[graphql(flatten)]
    construction_material: database::ConstructionMaterial,
}

impl From<database::ConstructionMaterial> for GQLConstructionMaterial {
    fn from(value: database::ConstructionMaterial) -> Self {
        GQLConstructionMaterial {
            construction_material: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLConstructionMaterial {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ConstructionShipment")]
#[graphql(complex)]
pub struct GQLConstructionShipment {
    #[graphql(flatten)]
    construction_shipment: database::ConstructionShipment,
}

impl From<database::ConstructionShipment> for GQLConstructionShipment {
    fn from(value: database::ConstructionShipment) -> Self {
        GQLConstructionShipment {
            construction_shipment: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLConstructionShipment {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Contract")]
#[graphql(complex)]
pub struct GQLContract {
    #[graphql(flatten)]
    contract: database::Contract,
}

impl From<database::Contract> for GQLContract {
    fn from(value: database::Contract) -> Self {
        GQLContract { contract: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLContract {
    async fn deliveries<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::ContractDelivery>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::ContractDelivery::get_by_contract_id(database_pool, &self.id).await
    }

    async fn shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::ContractShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ContractShipment::get_by_contract_id(database_pool, &self.id).await?;
        Ok(erg)
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::MarketTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::MarketTransaction::get_by_contract(database_pool, &self.id).await
    }

    async fn market_transaction_summary<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<database::TransactionSummary> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::MarketTransaction::get_transaction_summary_by_contract(database_pool, &self.id)
            .await
    }
}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ContractDelivery")]
#[graphql(complex)]
pub struct GQLContractDelivery {
    #[graphql(flatten)]
    contract_delivery: database::ContractDelivery,
}

impl From<database::ContractDelivery> for GQLContractDelivery {
    fn from(value: database::ContractDelivery) -> Self {
        GQLContractDelivery {
            contract_delivery: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLContractDelivery {
    async fn contract(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<database::Contract>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Contract::get_by_id(database_pool, &self.contract_id).await
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ContractShipment")]
#[graphql(complex)]
pub struct GQLContractShipment {
    #[graphql(flatten)]
    contract_shipment: database::ContractShipment,
}

impl From<database::ContractShipment> for GQLContractShipment {
    fn from(value: database::ContractShipment) -> Self {
        GQLContractShipment {
            contract_shipment: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLContractShipment {
    async fn contract<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Contract>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Contract::get_by_id(database_pool, &self.contract_id).await
    }

    async fn destination_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_symbol(database_pool, &self.destination_symbol).await
    }

    async fn purchase_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_symbol(database_pool, &self.purchase_symbol).await
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "EngineInfo")]
#[graphql(complex)]
pub struct GQLEngineInfo {
    #[graphql(flatten)]
    engine_info: database::EngineInfo,
}

impl From<database::EngineInfo> for GQLEngineInfo {
    fn from(value: database::EngineInfo) -> Self {
        GQLEngineInfo { engine_info: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLEngineInfo {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Extraction")]
#[graphql(complex)]
pub struct GQLExtraction {
    #[graphql(flatten)]
    extraction: database::Extraction,
}

impl From<database::Extraction> for GQLExtraction {
    fn from(value: database::Extraction) -> Self {
        GQLExtraction { extraction: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLExtraction {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await
    }

    #[graphql(name = "survey")]
    async fn get_survey<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Survey>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(survey) = &self.survey {
            let erg = database::Survey::get_by_signature(database_pool, survey).await?;
            Ok(Some(erg))
        } else {
            Ok(None)
        }
    }
}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Fleet")]
#[graphql(complex)]
pub struct GQLFleet {
    #[graphql(flatten)]
    fleet: database::Fleet,
}

impl From<database::Fleet> for GQLFleet {
    fn from(value: database::Fleet) -> Self {
        GQLFleet { fleet: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLFleet {
    async fn config(&self) -> Result<database::FleetConfig> {
        self.get_config()
    }

    async fn system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::System>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::System::get_by_symbol(database_pool, &self.system_symbol).await
    }

    async fn assignments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::ShipAssignment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::ShipAssignment::get_by_fleet_id(database_pool, self.id).await
    }
}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "FrameInfo")]
#[graphql(complex)]
pub struct GQLFrameInfo {
    #[graphql(flatten)]
    frame_info: database::FrameInfo,
}

impl From<database::FrameInfo> for GQLFrameInfo {
    fn from(value: database::FrameInfo) -> Self {
        GQLFrameInfo { frame_info: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLFrameInfo {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "JumpGateConnection")]
#[graphql(complex)]
pub struct GQLJumpGateConnection {
    #[graphql(flatten)]
    jump_gate_connection: database::JumpGateConnection,
}

impl From<database::JumpGateConnection> for GQLJumpGateConnection {
    fn from(value: database::JumpGateConnection) -> Self {
        GQLJumpGateConnection {
            jump_gate_connection: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLJumpGateConnection {
    async fn waypoint_from<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.from).await?;
        Ok(erg)
    }

    async fn waypoint_to<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_symbol(database_pool, &self.to).await
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MarketTrade")]
#[graphql(complex)]
pub struct GQLMarketTrade {
    #[graphql(flatten)]
    market_trade: database::MarketTrade,
}

impl From<database::MarketTrade> for GQLMarketTrade {
    fn from(value: database::MarketTrade) -> Self {
        GQLMarketTrade {
            market_trade: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLMarketTrade {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MarketTradeGood")]
#[graphql(complex)]
pub struct GQLMarketTradeGood {
    #[graphql(flatten)]
    market_trade_good: database::MarketTradeGood,
}

impl From<database::MarketTradeGood> for GQLMarketTradeGood {
    fn from(value: database::MarketTradeGood) -> Self {
        GQLMarketTradeGood {
            market_trade_good: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLMarketTradeGood {
    async fn waypoint(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::MarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::MarketTradeGood::get_history_by_waypoint_and_trade_symbol(
            database_pool,
            &self.waypoint_symbol,
            &self.symbol,
        )
        .await?;
        Ok(history)
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MarketTransaction")]
#[graphql(complex)]
pub struct GQLMarketTransaction {
    #[graphql(flatten)]
    market_transaction: database::MarketTransaction,
}

impl From<database::MarketTransaction> for GQLMarketTransaction {
    fn from(value: database::MarketTransaction) -> Self {
        GQLMarketTransaction {
            market_transaction: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLMarketTransaction {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await?;
        Ok(waypoint)
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ModuleInfo")]
#[graphql(complex)]
pub struct GQLModuleInfo {
    #[graphql(flatten)]
    module_info: database::ModuleInfo,
}

impl From<database::ModuleInfo> for GQLModuleInfo {
    fn from(value: database::ModuleInfo) -> Self {
        GQLModuleInfo { module_info: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLModuleInfo {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MountInfo")]
#[graphql(complex)]
pub struct GQLMountInfo {
    #[graphql(flatten)]
    mount_info: database::MountInfo,
}

impl From<database::MountInfo> for GQLMountInfo {
    fn from(value: database::MountInfo) -> Self {
        GQLMountInfo { mount_info: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLMountInfo {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ReactorInfo")]
#[graphql(complex)]
pub struct GQLReactorInfo {
    #[graphql(flatten)]
    reactor_info: database::ReactorInfo,
}

impl From<database::ReactorInfo> for GQLReactorInfo {
    fn from(value: database::ReactorInfo) -> Self {
        GQLReactorInfo {
            reactor_info: value,
        }
    }
}

#[async_graphql::ComplexObject]
impl GQLReactorInfo {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "RepairTransaction")]
#[graphql(complex)]
pub struct GQLRepairTransaction {
    #[graphql(flatten)]
    repair_transaction: database::RepairTransaction,
}

impl From<database::RepairTransaction> for GQLRepairTransaction {
    fn from(value: database::RepairTransaction) -> Self {
        GQLRepairTransaction {
            repair_transaction: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLRepairTransaction {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await?;
        Ok(waypoint)
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ReservedFund")]
#[graphql(complex)]
pub struct GQLReservedFund {
    #[graphql(flatten)]
    reserved_fund: database::ReservedFund,
}

impl From<database::ReservedFund> for GQLReservedFund {
    fn from(value: database::ReservedFund) -> Self {
        GQLReservedFund {
            reserved_fund: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLReservedFund {}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Route")]
#[graphql(complex)]
pub struct GQLRoute {
    #[graphql(flatten)]
    route: database::Route,
}

impl From<database::Route> for GQLRoute {
    fn from(value: database::Route) -> Self {
        GQLRoute { route: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLRoute {
    async fn waypoint_from<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.from).await?;
        Ok(erg)
    }

    async fn waypoint_to<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_symbol(database_pool, &self.to).await
    }
}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ScrapTransaction")]
#[graphql(complex)]
pub struct GQLScrapTransaction {
    #[graphql(flatten)]
    scrap_transaction: database::ScrapTransaction,
}

impl From<database::ScrapTransaction> for GQLScrapTransaction {
    fn from(value: database::ScrapTransaction) -> Self {
        GQLScrapTransaction {
            scrap_transaction: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLScrapTransaction {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await?;
        Ok(waypoint)
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipAssignment")]
#[graphql(complex)]
pub struct GQLShipAssignment {
    #[graphql(flatten)]
    ship_assignment: database::ShipAssignment,
}

impl From<database::ShipAssignment> for GQLShipAssignment {
    fn from(value: database::ShipAssignment) -> Self {
        GQLShipAssignment {
            ship_assignment: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLShipAssignment {
    async fn fleet<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Fleet>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Fleet::get_by_id(database_pool, self.fleet_id).await
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipInfo")]
#[graphql(complex)]
pub struct GQLShipInfo {
    #[graphql(flatten)]
    ship_info: database::ShipInfo,
}

impl From<database::ShipInfo> for GQLShipInfo {
    fn from(value: database::ShipInfo) -> Self {
        GQLShipInfo { ship_info: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLShipInfo {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipJump")]
#[graphql(complex)]
pub struct GQLShipJump {
    #[graphql(flatten)]
    ship_jump: database::ShipJump,
}

impl From<database::ShipJump> for GQLShipJump {
    fn from(value: database::ShipJump) -> Self {
        GQLShipJump { ship_jump: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLShipJump {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipModificationTransaction")]
#[graphql(complex)]
pub struct GQLShipModificationTransaction {
    #[graphql(flatten)]
    ship_modification_transaction: database::ShipModificationTransaction,
}

impl From<database::ShipModificationTransaction> for GQLShipModificationTransaction {
    fn from(value: database::ShipModificationTransaction) -> Self {
        GQLShipModificationTransaction {
            ship_modification_transaction: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLShipModificationTransaction {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await?;
        Ok(waypoint)
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipState")]
#[graphql(complex)]
pub struct GQLShipState {
    #[graphql(flatten)]
    ship_state: database::ShipState,
}

impl From<database::ShipState> for GQLShipState {
    fn from(value: database::ShipState) -> Self {
        GQLShipState { ship_state: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLShipState {
    async fn cargo_inventory(&self) -> HashMap<models::TradeSymbol, i32> {
        self.ship_state.cargo_inventory.0.clone()
    }
}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Shipyard")]
#[graphql(complex)]
pub struct GQLShipyard {
    #[graphql(flatten)]
    shipyard: database::Shipyard,
}

impl From<database::Shipyard> for GQLShipyard {
    fn from(value: database::Shipyard) -> Self {
        GQLShipyard { shipyard: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLShipyard {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await
    }

    async fn history(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<database::Shipyard>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::Shipyard::get_history_by_waypoint(database_pool, &self.waypoint_symbol)
                .await?;
        Ok(history)
    }

    async fn shipyard_ship_types(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::ShipyardShipTypes>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ShipyardShipTypes::get_last_by_waypoint(database_pool, &self.waypoint_symbol)
                .await?;
        Ok(history)
    }

    async fn shipyard_ships(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::ShipyardShip>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ShipyardShip::get_last_by_waypoint(database_pool, &self.waypoint_symbol)
                .await?;
        Ok(history)
    }

    async fn shipyard_transactions(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::ShipyardTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ShipyardTransaction::get_by_waypoint(database_pool, &self.waypoint_symbol)
                .await?;
        Ok(history)
    }
}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipyardShip")]
#[graphql(complex)]
pub struct GQLShipyardShip {
    #[graphql(flatten)]
    shipyard_ship: database::ShipyardShip,
}

impl From<database::ShipyardShip> for GQLShipyardShip {
    fn from(value: database::ShipyardShip) -> Self {
        GQLShipyardShip {
            shipyard_ship: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLShipyardShip {
    async fn waypoint(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::ShipyardShip>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShip::get_history_by_waypoint_and_ship_type(
            database_pool,
            &self.waypoint_symbol,
            &self.ship_type,
        )
        .await?;
        Ok(history)
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipyardShipTypes")]
#[graphql(complex)]
pub struct GQLShipyardShipTypes {
    #[graphql(flatten)]
    shipyard_ship_types: database::ShipyardShipTypes,
}

impl From<database::ShipyardShipTypes> for GQLShipyardShipTypes {
    fn from(value: database::ShipyardShipTypes) -> Self {
        GQLShipyardShipTypes {
            shipyard_ship_types: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLShipyardShipTypes {}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipyardTransaction")]
#[graphql(complex)]
pub struct GQLShipyardTransaction {
    #[graphql(flatten)]
    shipyard_transaction: database::ShipyardTransaction,
}

impl From<database::ShipyardTransaction> for GQLShipyardTransaction {
    fn from(value: database::ShipyardTransaction) -> Self {
        GQLShipyardTransaction {
            shipyard_transaction: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLShipyardTransaction {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await?;
        Ok(waypoint)
    }

    async fn agent<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Agent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let agent = database::Agent::get_last_by_symbol(database_pool, &self.agent_symbol).await?;
        Ok(agent)
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Survey")]
#[graphql(complex)]
pub struct GQLSurvey {
    #[graphql(flatten)]
    survey: database::Survey,
}

impl From<database::Survey> for GQLSurvey {
    fn from(value: database::Survey) -> Self {
        GQLSurvey { survey: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLSurvey {
    async fn percent(&self) -> Vec<SurveyPercent> {
        self.get_percent()
            .iter()
            .map(|f| SurveyPercent {
                symbol: f.0,
                percent: f.1,
            })
            .collect()
    }

    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_symbol(database_pool, &self.waypoint_symbol).await
    }

    async fn extractions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::Extraction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Extraction::get_by_survey_symbol(database_pool, &self.signature).await
    }
}

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
pub struct SurveyPercent {
    pub symbol: models::TradeSymbol,
    pub percent: f64,
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "System")]
#[graphql(complex)]
pub struct GQLSystem {
    #[graphql(flatten)]
    system: database::System,
}

impl From<database::System> for GQLSystem {
    fn from(value: database::System) -> Self {
        GQLSystem { system: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLSystem {
    async fn waypoints(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<database::Waypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        database::Waypoint::get_by_system(database_pool, &self.symbol).await
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::MarketTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::MarketTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn shipyard_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::ShipyardTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::ShipyardTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn chart_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::ChartTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::ChartTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn repair_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::RepairTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::RepairTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn scrap_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::ScrapTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::ScrapTransaction::get_by_system(database_pool, &self.symbol).await?;
        Ok(transactions)
    }
    async fn ship_modification_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<database::ShipModificationTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::ShipModificationTransaction::get_by_system(database_pool, &self.symbol)
                .await?;
        Ok(transactions)
    }

    async fn shipyard_ships(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::ShipyardShip>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ShipyardShip::get_last_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn shipyard_ship_types(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::ShipyardShipTypes>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ShipyardShipTypes::get_last_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn market_trades(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::MarketTrade>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::MarketTrade::get_last_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn market_trade_goods(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::MarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::MarketTradeGood::get_last_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn fleets(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<database::Fleet>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::Fleet::get_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn surveys(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<database::Survey>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::Survey::get_by_system_symbol(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn extractions(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::Extraction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::Extraction::get_by_system_symbol(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn construction_materials(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::ConstructionMaterial>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ConstructionMaterial::get_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn construction_shipments(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::ConstructionShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ConstructionShipment::get_by_system(database_pool, &self.symbol).await?;
        Ok(history)
    }

    async fn contract_deliveries(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<database::ContractDelivery>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ContractDelivery::get_by_system_symbol(database_pool, &self.symbol).await?;
        Ok(history)
    }

    // async fn contract_shipments(
    //     &self,
    //     ctx: &async_graphql::Context<'_>,
    // ) -> Result<Vec<database::ContractShipment>> {
    //     let database_pool = ctx.data::<database::DbPool>().unwrap();
    //     let history =
    //         database::ContractShipment::get_by_system_symbol(database_pool, &self.symbol).await?;
    //     Ok(history)
    // }

    async fn seen_agents(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<KnownAgent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system_market_transactions =
            database::MarketTransaction::get_by_system(database_pool, &self.symbol).await?;

        let system_shipyard_transactions =
            database::ShipyardTransaction::get_by_system(database_pool, &self.symbol).await?;

        let known_agents_iter = system_market_transactions
            .iter()
            .filter_map(|f| {
                f.ship_symbol
                    .chars()
                    .rev()
                    .collect::<String>()
                    .split_once("-")
                    .map(|f| f.1.chars().rev().collect::<String>())
            })
            .chain(
                system_shipyard_transactions
                    .iter()
                    .map(|f| f.agent_symbol.clone()),
            );

        let known_agents = known_agents_iter
            .fold(std::collections::HashMap::new(), |mut acc, f| {
                acc.entry(f).and_modify(|e: &mut u32| *e += 1).or_insert(1);
                acc
            })
            .into_iter()
            .map(|f| KnownAgent {
                symbol: f.0,
                count: f.1,
            })
            .collect();
        Ok(known_agents)
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
struct KnownAgent {
    symbol: String,
    count: u32,
}

#[async_graphql::ComplexObject]
impl KnownAgent {
    async fn agent(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<database::Agent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let agent = database::Agent::get_last_by_symbol(database_pool, &self.symbol).await?;
        Ok(agent)
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "TradeRoute")]
#[graphql(complex)]
pub struct GQLTradeRoute {
    #[graphql(flatten)]
    trade_route: database::TradeRoute,
}

impl From<database::TradeRoute> for GQLTradeRoute {
    fn from(value: database::TradeRoute) -> Self {
        GQLTradeRoute { trade_route: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLTradeRoute {}
#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Waypoint")]
#[graphql(complex)]
pub struct GQLWaypoint {
    #[graphql(flatten)]
    waypoint: database::Waypoint,
}

impl From<database::Waypoint> for GQLWaypoint {
    fn from(value: database::Waypoint) -> Self {
        GQLWaypoint { waypoint: value }
    }
}
#[async_graphql::ComplexObject]
impl GQLWaypoint {}
