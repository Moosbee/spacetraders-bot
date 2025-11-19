#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Agent")]
pub struct GQLAgent {
    #[graphql(flatten)]
    agent: database::Agent,
}

impl From<database::Agent> for GQLAgent {
    fn from(value: database::Agent) -> Self {
        GQLAgent { agent: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ChartTransaction")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ConstructionMaterial")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ConstructionShipment")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Contract")]
pub struct GQLContract {
    #[graphql(flatten)]
    contract: database::Contract,
}

impl From<database::Contract> for GQLContract {
    fn from(value: database::Contract) -> Self {
        GQLContract { contract: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ContractDelivery")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ContractShipment")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "EngineInfo")]
pub struct GQLEngineInfo {
    #[graphql(flatten)]
    engine_info: database::EngineInfo,
}

impl From<database::EngineInfo> for GQLEngineInfo {
    fn from(value: database::EngineInfo) -> Self {
        GQLEngineInfo { engine_info: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Extraction")]
pub struct GQLExtraction {
    #[graphql(flatten)]
    extraction: database::Extraction,
}

impl From<database::Extraction> for GQLExtraction {
    fn from(value: database::Extraction) -> Self {
        GQLExtraction { extraction: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Fleet")]
pub struct GQLFleet {
    #[graphql(flatten)]
    fleet: database::Fleet,
}

impl From<database::Fleet> for GQLFleet {
    fn from(value: database::Fleet) -> Self {
        GQLFleet { fleet: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "FrameInfo")]
pub struct GQLFrameInfo {
    #[graphql(flatten)]
    frame_info: database::FrameInfo,
}

impl From<database::FrameInfo> for GQLFrameInfo {
    fn from(value: database::FrameInfo) -> Self {
        GQLFrameInfo { frame_info: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "JumpGateConnection")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MarketTrade")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MarketTradeGood")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MarketTransaction")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ModuleInfo")]
pub struct GQLModuleInfo {
    #[graphql(flatten)]
    module_info: database::ModuleInfo,
}

impl From<database::ModuleInfo> for GQLModuleInfo {
    fn from(value: database::ModuleInfo) -> Self {
        GQLModuleInfo { module_info: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MountInfo")]
pub struct GQLMountInfo {
    #[graphql(flatten)]
    mount_info: database::MountInfo,
}

impl From<database::MountInfo> for GQLMountInfo {
    fn from(value: database::MountInfo) -> Self {
        GQLMountInfo { mount_info: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ReactorInfo")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "RepairTransaction")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ReservedFund")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Route")]
pub struct GQLRoute {
    #[graphql(flatten)]
    route: database::Route,
}

impl From<database::Route> for GQLRoute {
    fn from(value: database::Route) -> Self {
        GQLRoute { route: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ScrapTransaction")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipAssignment")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipInfo")]
pub struct GQLShipInfo {
    #[graphql(flatten)]
    ship_info: database::ShipInfo,
}

impl From<database::ShipInfo> for GQLShipInfo {
    fn from(value: database::ShipInfo) -> Self {
        GQLShipInfo { ship_info: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipModificationTransaction")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipState")]
pub struct GQLShipState {
    #[graphql(flatten)]
    ship_state: database::ShipState,
}

impl From<database::ShipState> for GQLShipState {
    fn from(value: database::ShipState) -> Self {
        GQLShipState { ship_state: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipJump")]
pub struct GQLShipJump {
    #[graphql(flatten)]
    ship_jump: database::ShipJump,
}

impl From<database::ShipJump> for GQLShipJump {
    fn from(value: database::ShipJump) -> Self {
        GQLShipJump { ship_jump: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Shipyard")]
pub struct GQLShipyard {
    #[graphql(flatten)]
    shipyard: database::Shipyard,
}

impl From<database::Shipyard> for GQLShipyard {
    fn from(value: database::Shipyard) -> Self {
        GQLShipyard { shipyard: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipyardShip")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipyardShipTypes")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipyardTransaction")]
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Survey")]
pub struct GQLSurvey {
    #[graphql(flatten)]
    survey: database::Survey,
}

impl From<database::Survey> for GQLSurvey {
    fn from(value: database::Survey) -> Self {
        GQLSurvey { survey: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "System")]
pub struct GQLSystem {
    #[graphql(flatten)]
    system: database::System,
}

impl From<database::System> for GQLSystem {
    fn from(value: database::System) -> Self {
        GQLSystem { system: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "TradeRoute")]
pub struct GQLTradeRoute {
    #[graphql(flatten)]
    trade_route: database::TradeRoute,
}

impl From<database::TradeRoute> for GQLTradeRoute {
    fn from(value: database::TradeRoute) -> Self {
        GQLTradeRoute { trade_route: value }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Waypoint")]
pub struct GQLWaypoint {
    #[graphql(flatten)]
    waypoint: database::Waypoint,
}

impl From<database::Waypoint> for GQLWaypoint {
    fn from(value: database::Waypoint) -> Self {
        GQLWaypoint { waypoint: value }
    }
}
