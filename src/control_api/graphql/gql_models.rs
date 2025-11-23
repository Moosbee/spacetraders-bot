use std::collections::HashMap;

use ship::status::ShipStatus;
use space_traders_client::models;

use crate::{
    control_api::graphql::gql_ship::{GQLModules, GQLMounts, GQLNavigationState},
    error::Result,
};

/// Utility function to convert a single optional database type to its GQL type.
fn into_gql<Db, Gql>(db: Option<Db>) -> Option<Gql>
where
    Gql: From<Db>,
{
    db.map(Gql::from)
}

/// Utility function to convert a vector of database types to a vector of their GQL types.
fn into_gql_vec<Db, Gql>(db_vec: Vec<Db>) -> Vec<Gql>
where
    Gql: From<Db>,
{
    db_vec.into_iter().map(Gql::from).collect()
}

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
    async fn history<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLAgent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::Agent::get_by_symbol(database_pool, &self.agent.symbol).await?;
        Ok(into_gql_vec(reg))
    }

    async fn headquarters_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::Waypoint::get_by_symbol(database_pool, &self.agent.headquarters).await?;
        Ok(into_gql(reg))
    }

    async fn headquarters_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = utils::get_system_symbol(&self.agent.headquarters);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(into_gql(erg))
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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.chart_transaction.waypoint_symbol,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.chart_transaction.ship_symbol);
        Ok(ship.map(|f| f.into()))
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
impl GQLConstructionMaterial {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.construction_material.waypoint_symbol,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    async fn market_transaction_summary<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<GQLTransactionSummary> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTransaction::get_transaction_summary_by_construction_material(
            database_pool,
            self.construction_material.id,
        )
        .await?;
        Ok(GQLTransactionSummary::from(erg))
    }

    async fn shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLConstructionShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ConstructionShipment::get_by_material_id(
            database_pool,
            self.construction_material.id,
        )
        .await?;
        Ok(into_gql_vec(erg))
    }
}

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
impl GQLConstructionShipment {
    async fn construction_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.construction_shipment.construction_site_waypoint,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    async fn purchase_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.construction_shipment.purchase_waypoint,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    async fn market_transaction_summary<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<GQLTransactionSummary> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTransaction::get_transaction_summary_by_construction_shipment(
            database_pool,
            self.construction_shipment.id,
        )
        .await?;
        Ok(GQLTransactionSummary::from(erg))
    }

    async fn material(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLConstructionMaterial>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let material = database::ConstructionMaterial::get_by_id(
            database_pool,
            self.construction_shipment.material_id,
        )
        .await?;
        Ok(material.map(|f| f.into()))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.construction_shipment.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }

    async fn reservation<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLReservedFund>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = if let Some(reserved_fund) = self.construction_shipment.reserved_fund {
            database::ReservedFund::get_by_id(database_pool, &reserved_fund).await?
        } else {
            None
        };
        Ok(erg.map(|f| f.into()))
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLMarketTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTransaction::get_by_construction(
            database_pool,
            self.construction_shipment.id,
        )
        .await?;
        Ok(into_gql_vec(erg))
    }
}

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
    ) -> Result<Vec<GQLContractDelivery>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ContractDelivery::get_by_contract_id(database_pool, &self.contract.id)
            .await?;
        Ok(into_gql_vec(erg))
    }

    async fn shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLContractShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ContractShipment::get_by_contract_id(database_pool, &self.contract.id)
            .await?;
        Ok(into_gql_vec(erg))
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLMarketTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::MarketTransaction::get_by_contract(database_pool, &self.contract.id).await?;
        Ok(into_gql_vec(erg))
    }

    async fn market_transaction_summary<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<GQLTransactionSummary> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTransaction::get_transaction_summary_by_contract(
            database_pool,
            &self.contract.id,
        )
        .await?;
        Ok(GQLTransactionSummary::from(erg))
    }

    async fn reservation<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLReservedFund>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = if let Some(reserved_fund) = self.contract.reserved_fund {
            database::ReservedFund::get_by_id(database_pool, &reserved_fund).await?
        } else {
            None
        };
        Ok(erg.map(|f| f.into()))
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "TransactionSummary")]
#[graphql(complex)]
pub struct GQLTransactionSummary {
    #[graphql(flatten)]
    transaction_summary: database::TransactionSummary,
}

impl From<database::TransactionSummary> for GQLTransactionSummary {
    fn from(value: database::TransactionSummary) -> Self {
        GQLTransactionSummary {
            transaction_summary: value,
        }
    }
}
#[async_graphql::ComplexObject]
impl GQLTransactionSummary {}

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
    async fn contract(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLContract>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Contract::get_by_id(database_pool, &self.contract_delivery.contract_id)
            .await?;
        Ok(into_gql(erg))
    }

    async fn contract_shipment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLContractShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ContractShipment::get_by_contract_id_trade_symbol_destination_symbol(
            database_pool,
            &self.contract_delivery.contract_id,
            &self.contract_delivery.trade_symbol,
            &self.contract_delivery.destination_symbol,
        )
        .await?;
        Ok(into_gql_vec(erg))
    }

    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(
            database_pool,
            &self.contract_delivery.destination_symbol,
        )
        .await?;
        Ok(into_gql(erg))
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
    ) -> Result<Option<GQLContract>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Contract::get_by_id(database_pool, &self.contract_shipment.contract_id)
            .await?;
        Ok(into_gql(erg))
    }

    async fn destination_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(
            database_pool,
            &self.contract_shipment.destination_symbol,
        )
        .await?;
        Ok(into_gql(erg))
    }

    async fn purchase_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(
            database_pool,
            &self.contract_shipment.purchase_symbol,
        )
        .await?;
        Ok(into_gql(erg))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.contract_shipment.ship_symbol);
        Ok(ship.map(|f| f.into()))
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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::Waypoint::get_by_symbol(database_pool, &self.extraction.waypoint_symbol)
                .await?;
        Ok(into_gql(erg))
    }

    async fn survey<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLSurvey>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(survey) = &self.extraction.survey {
            let erg = database::Survey::get_by_signature(database_pool, survey).await?;
            // Must convert before wrapping in Option
            Ok(Some(GQLSurvey::from(erg)))
        } else {
            Ok(None)
        }
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context.ship_manager.get_clone(&self.extraction.ship_symbol);
        Ok(ship.map(|f| f.into()))
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
        let erg: database::FleetConfig = self.fleet.get_config()?;
        Ok(erg)
    }

    async fn system<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::System::get_by_symbol(database_pool, &self.fleet.system_symbol).await?;
        Ok(into_gql(erg))
    }

    async fn assignments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLShipAssignment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ShipAssignment::get_by_fleet_id(database_pool, self.fleet.id).await?;
        Ok(into_gql_vec(erg))
    }

    async fn all_ships<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let all_ships = context.ship_manager.get_all_clone().await;
        let ships = all_ships
            .into_values()
            .filter(|ship| {
                ship.status.fleet_id == Some(self.fleet.id)
                    || ship.status.temp_fleet_id == Some(self.fleet.id)
            })
            .collect::<Vec<_>>();
        Ok(ships.into_iter().map(|s| s.into()).collect())
    }
    async fn ships<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let all_ships = context.ship_manager.get_all_clone().await;
        let ships = all_ships
            .into_values()
            .filter(|ship| ship.status.fleet_id == Some(self.fleet.id))
            .collect::<Vec<_>>();
        Ok(ships.into_iter().map(|s| s.into()).collect())
    }
    async fn temp_ships<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let all_ships = context.ship_manager.get_all_clone().await;
        let ships = all_ships
            .into_values()
            .filter(|ship| ship.status.temp_fleet_id == Some(self.fleet.id))
            .collect::<Vec<_>>();
        Ok(ships.into_iter().map(|s| s.into()).collect())
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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.jump_gate_connection.from)
            .await?;
        Ok(into_gql(erg))
    }

    async fn waypoint_to<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::Waypoint::get_by_symbol(database_pool, &self.jump_gate_connection.to).await?;
        Ok(into_gql(erg))
    }

    async fn system_from<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = utils::get_system_symbol(&self.jump_gate_connection.from);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(into_gql(erg))
    }

    async fn system_to<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = utils::get_system_symbol(&self.jump_gate_connection.to);
        let erg = database::System::get_by_symbol(database_pool, &system).await?;
        Ok(into_gql(erg))
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
impl GQLMarketTrade {
    async fn waypoint(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::Waypoint::get_by_symbol(database_pool, &self.market_trade.waypoint_symbol)
                .await?;
        Ok(into_gql(erg))
    }

    async fn history(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLMarketTrade>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTrade::get_history_by_waypoint_and_trade_symbol(
            database_pool,
            &self.market_trade.waypoint_symbol,
            &self.market_trade.symbol,
        )
        .await?;
        Ok(erg.into_iter().map(|t| t.into()).collect())
    }

    async fn market_trade_good(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLMarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTradeGood::get_by_last_waypoint_and_trade_symbol(
            database_pool,
            &self.market_trade.waypoint_symbol,
            &self.market_trade.symbol,
        )
        .await?;
        Ok(into_gql(erg))
    }
}

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
    async fn waypoint(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(
            database_pool,
            &self.market_trade_good.waypoint_symbol,
        )
        .await?;
        Ok(into_gql(erg))
    }

    async fn history(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLMarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::MarketTradeGood::get_history_by_waypoint_and_trade_symbol(
            database_pool,
            &self.market_trade_good.waypoint_symbol,
            &self.market_trade_good.symbol,
        )
        .await?;
        Ok(into_gql_vec(history))
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLMarketTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTransaction::get_by_waypoint_and_trade_symbol(
            database_pool,
            &self.market_trade_good.waypoint_symbol,
            self.market_trade_good.symbol,
        )
        .await?;
        Ok(erg.into_iter().map(|t| t.into()).collect())
    }

    async fn market_trade(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLMarketTrade>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTrade::get_by_last_waypoint_and_trade_symbol(
            database_pool,
            &self.market_trade_good.waypoint_symbol,
            &self.market_trade_good.symbol,
        )
        .await?;
        Ok(into_gql(erg))
    }

    async fn market_transaction_summary<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<GQLTransactionSummary> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::MarketTransaction::get_transaction_summary_by_waypoint_and_trade_symbol(
                database_pool,
                &self.market_trade_good.waypoint_symbol,
                self.market_trade_good.symbol,
            )
            .await?;
        Ok(erg.into())
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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.market_transaction.waypoint_symbol,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    pub async fn ship<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.market_transaction.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }

    pub async fn contract(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLContract>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let contract = if let Some(contract) = self.market_transaction.contract.clone() {
            database::Contract::get_by_id(database_pool, &contract).await?
        } else {
            None
        };
        Ok(into_gql(contract))
    }

    pub async fn trade_route<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLTradeRoute>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let trade_route = if let Some(trade_route) = self.market_transaction.trade_route {
            database::TradeRoute::get_by_id(database_pool, trade_route).await?
        } else {
            None
        };
        Ok(into_gql(trade_route))
    }

    pub async fn mining_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = if let Some(waypoint) = self.market_transaction.mining.clone() {
            database::Waypoint::get_by_symbol(database_pool, &waypoint).await?
        } else {
            None
        };
        Ok(into_gql(waypoint))
    }

    pub async fn construction_shipment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLConstructionShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let construction_shipment = if let Some(construction_shipment) =
            self.market_transaction.construction
        {
            database::ConstructionShipment::get_by_id(database_pool, construction_shipment).await?
        } else {
            None
        };
        Ok(into_gql(construction_shipment))
    }

    pub async fn market_trade_good<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLMarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let trade_good = database::MarketTradeGood::get_by_last_waypoint_and_trade_symbol(
            database_pool,
            &self.market_transaction.waypoint_symbol,
            &self.market_transaction.trade_symbol,
        )
        .await?;
        Ok(into_gql(trade_good))
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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.repair_transaction.waypoint_symbol,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.repair_transaction.ship_symbol);
        Ok(ship.map(|f| f.into()))
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
impl GQLReservedFund {
    async fn contract<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLContract>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let contract =
            database::Contract::get_by_reservation_id(database_pool, self.reserved_fund.id).await?;
        Ok(into_gql_vec(contract))
    }

    async fn trade_route<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLTradeRoute>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let trade_routes =
            database::TradeRoute::get_by_reservation_id(database_pool, self.reserved_fund.id)
                .await?;
        Ok(into_gql_vec(trade_routes))
    }

    async fn construction_shipment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLConstructionShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let construction_shipments = database::ConstructionShipment::get_by_reservation_id(
            database_pool,
            self.reserved_fund.id,
        )
        .await?;
        Ok(into_gql_vec(construction_shipments))
    }
}

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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.route.from).await?;
        Ok(into_gql(erg))
    }

    async fn waypoint_to<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.route.to).await?;
        Ok(into_gql(erg))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context.ship_manager.get_clone(&self.route.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }

    async fn ship_state_after<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShipState>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let ship_state = if let Some(ship_state) = &self.route.ship_info_after {
            database::ShipState::get_by_id(database_pool, *ship_state).await?
        } else {
            None
        };
        Ok(into_gql(ship_state))
    }

    async fn ship_state_before<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShipState>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let ship_state = if let Some(ship_state) = &self.route.ship_info_before {
            database::ShipState::get_by_id(database_pool, *ship_state).await?
        } else {
            None
        };
        Ok(into_gql(ship_state))
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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.scrap_transaction.waypoint_symbol,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.scrap_transaction.ship_symbol);
        Ok(ship.map(|f| f.into()))
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
    async fn fleet<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLFleet>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Fleet::get_by_id(database_pool, self.ship_assignment.fleet_id).await?;
        Ok(into_gql(erg))
    }

    async fn all_ship<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let all_ships = context.ship_manager.get_all_clone().await;
        let ship = all_ships.into_values().find(|ship| {
            ship.status.fleet_id == Some(self.ship_assignment.fleet_id)
                || ship.status.temp_fleet_id == Some(self.ship_assignment.fleet_id)
        });
        Ok(ship.map(|f| f.into()))
    }

    async fn ship<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let all_ships = context.ship_manager.get_all_clone().await;
        let ship = all_ships
            .into_values()
            .find(|ship| ship.status.fleet_id == Some(self.ship_assignment.fleet_id));
        Ok(ship.map(|f| f.into()))
    }

    async fn temp_ship<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let all_ships = context.ship_manager.get_all_clone().await;
        let ship = all_ships
            .into_values()
            .find(|ship| ship.status.temp_fleet_id == Some(self.ship_assignment.fleet_id));
        Ok(ship.map(|f| f.into()))
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
impl GQLShipInfo {
    async fn ship<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context.ship_manager.get_clone(&self.ship_info.symbol);
        Ok(ship.map(|f| f.into()))
    }

    // async fn fleet<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLFleet>> {
    //     let database_pool = ctx.data::<database::DbPool>().unwrap();
    //     let erg = database::Fleet::get_by_id(database_pool, self.ship_info.fleet_id).await?;
    //     Ok(into_gql(erg))
    // }

    // async fn temp_fleet<'ctx>(
    //     &self,
    //     ctx: &async_graphql::Context<'ctx>,
    // ) -> Result<Option<GQLFleet>> {
    //     let database_pool = ctx.data::<database::DbPool>().unwrap();
    //     let erg = database::Fleet::get_by_id(database_pool, self.ship_info.temp_fleet_id).await?;
    //     Ok(into_gql(erg))
    // }

    async fn assignment(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLShipAssignment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = if let Some(assignment_id) = self.ship_info.assignment_id {
            database::ShipAssignment::get_by_id(database_pool, assignment_id).await?
        } else {
            None
        };
        Ok(into_gql(erg))
    }

    async fn temp_assignment(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLShipAssignment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = if let Some(assignment_id) = self.ship_info.temp_assignment_id {
            database::ShipAssignment::get_by_id(database_pool, assignment_id).await?
        } else {
            None
        };
        Ok(into_gql(erg))
    }

    async fn purchase_transaction<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShipyardTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = if let Some(transaction_id) = self.ship_info.purchase_id {
            Some(database::ShipyardTransaction::get_by_id(database_pool, transaction_id).await?)
        } else {
            None
        };
        Ok(into_gql(erg))
    }
}

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
impl GQLShipJump {
    async fn waypoint_from<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.ship_jump.from).await?;
        Ok(into_gql(erg))
    }

    async fn waypoint_to<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.ship_jump.to).await?;
        Ok(into_gql(erg))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context.ship_manager.get_clone(&self.ship_jump.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }

    async fn ship_state_after<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShipState>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let ship_state =
            database::ShipState::get_by_id(database_pool, self.ship_jump.ship_after).await?;
        Ok(into_gql(ship_state))
    }

    async fn ship_state_before<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShipState>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let ship_state =
            database::ShipState::get_by_id(database_pool, self.ship_jump.ship_before).await?;
        Ok(into_gql(ship_state))
    }
}

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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.ship_modification_transaction.waypoint_symbol,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.ship_modification_transaction.ship_symbol);
        Ok(ship.map(|f| f.into()))
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

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context.ship_manager.get_clone(&self.ship_state.symbol);
        Ok(ship.map(|f| f.into()))
    }

    // getters(from db) for system_symbol,waypoint_symbol,route_destination_symbol,route_destination_system,route_origin_symbol,route_origin_system,auto_pilot_destination_symbol,auto_pilot_destination_system_symbol,auto_pilot_origin_symbol,auto_pilot_origin_system_symbol
    async fn system_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system =
            database::System::get_by_symbol(database_pool, &self.ship_state.system_symbol).await?;
        Ok(into_gql(system))
    }

    async fn waypoint_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.ship_state.waypoint_symbol)
                .await?;
        Ok(into_gql(waypoint))
    }

    async fn route_destination_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.ship_state.route_destination_symbol,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    async fn route_destination_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = database::System::get_by_symbol(
            database_pool,
            &self.ship_state.route_destination_system,
        )
        .await?;
        Ok(into_gql(system))
    }

    async fn route_origin_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.ship_state.route_origin_symbol)
                .await?;
        Ok(into_gql(waypoint))
    }

    async fn route_origin_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system =
            database::System::get_by_symbol(database_pool, &self.ship_state.route_origin_system)
                .await?;
        Ok(into_gql(system))
    }

    async fn auto_pilot_destination_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = if let Some(ap_dest) = &self.ship_state.auto_pilot_destination_symbol {
            database::Waypoint::get_by_symbol(database_pool, ap_dest).await?
        } else {
            None
        };
        Ok(into_gql(waypoint))
    }

    async fn auto_pilot_destination_system_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = if let Some(ap_dest) = &self.ship_state.auto_pilot_destination_system_symbol {
            database::System::get_by_symbol(database_pool, ap_dest).await?
        } else {
            None
        };
        Ok(into_gql(system))
    }

    async fn auto_pilot_origin_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = if let Some(ap_orig) = &self.ship_state.auto_pilot_origin_symbol {
            database::Waypoint::get_by_symbol(database_pool, ap_orig).await?
        } else {
            None
        };
        Ok(into_gql(waypoint))
    }

    async fn auto_pilot_origin_system_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = if let Some(ap_orig) = &self.ship_state.auto_pilot_origin_system_symbol {
            database::System::get_by_symbol(database_pool, ap_orig).await?
        } else {
            None
        };
        Ok(into_gql(system))
    }

    async fn engine_info<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<GQLEngineInfo> {
        // Changed return type to GQLEngineInfo
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::EngineInfo::get_by_symbol(database_pool, &self.ship_state.engine_symbol)
                .await?;
        Ok(GQLEngineInfo::from(reg)) // Added conversion
    }

    async fn frame_info<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<GQLFrameInfo> {
        // Changed return type to GQLFrameInfo
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::FrameInfo::get_by_symbol(database_pool, &self.ship_state.frame_symbol)
            .await?;
        Ok(GQLFrameInfo::from(reg)) // Added conversion
    }

    async fn reactor_info<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<GQLReactorInfo> {
        // Changed return type to GQLReactorInfo
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::ReactorInfo::get_by_symbol(database_pool, &self.ship_state.reactor_symbol)
                .await?;
        Ok(GQLReactorInfo::from(reg)) // Added conversion
    }

    async fn module_infos<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLModuleInfo>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let mut modules = Vec::new();
        for module_symbol in self.ship_state.modules.iter() {
            let erg = database::ModuleInfo::get_by_id(database_pool, module_symbol).await?;
            modules.push(erg);
        }
        Ok(modules.into_iter().map(|m| m.into()).collect())
    }

    async fn mount_infos<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLMountInfo>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let mut mounts = Vec::new();
        for mount_symbol in self.ship_state.mounts.iter() {
            let erg = database::MountInfo::get_by_id(database_pool, mount_symbol).await?;
            mounts.push(erg);
        }
        Ok(mounts.into_iter().map(|m| m.into()).collect())
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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_symbol(database_pool, &self.shipyard.waypoint_symbol)
            .await?;
        Ok(into_gql(erg))
    }

    async fn history(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLShipyard>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::Shipyard::get_history_by_waypoint(
            database_pool,
            &self.shipyard.waypoint_symbol,
        )
        .await?;
        Ok(into_gql_vec(history))
    }

    async fn shipyard_ship_types(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLShipyardShipTypes>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShipTypes::get_last_by_waypoint(
            database_pool,
            &self.shipyard.waypoint_symbol,
        )
        .await?;
        Ok(into_gql_vec(history))
    }

    async fn shipyard_ships(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLShipyardShip>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShip::get_last_by_waypoint(
            database_pool,
            &self.shipyard.waypoint_symbol,
        )
        .await?;
        Ok(into_gql_vec(history))
    }

    async fn shipyard_transactions(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLShipyardTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardTransaction::get_by_waypoint(
            database_pool,
            &self.shipyard.waypoint_symbol,
        )
        .await?;
        Ok(into_gql_vec(history))
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
    async fn waypoint(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::Waypoint::get_by_symbol(database_pool, &self.shipyard_ship.waypoint_symbol)
                .await?;
        Ok(into_gql(erg))
    }

    async fn history(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLShipyardShip>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShip::get_history_by_waypoint_and_ship_type(
            database_pool,
            &self.shipyard_ship.waypoint_symbol,
            &self.shipyard_ship.ship_type,
        )
        .await?;
        Ok(into_gql_vec(history))
    }

    async fn shipyard_transactions(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLShipyardTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardTransaction::get_by_waypoint_and_ship_type(
            database_pool,
            &self.shipyard_ship.waypoint_symbol,
            self.shipyard_ship.ship_type,
        )
        .await?;
        Ok(into_gql_vec(history))
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
impl GQLShipyardShipTypes {
    async fn shipyard(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShipyard>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::Shipyard::get_by_id(database_pool, self.shipyard_ship_types.shipyard_id)
                .await?;
        Ok(into_gql(erg))
    }
}

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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(
            database_pool,
            &self.shipyard_transaction.waypoint_symbol,
        )
        .await?;
        Ok(into_gql(waypoint))
    }

    async fn agent<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLAgent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let agent = database::Agent::get_last_by_symbol(
            database_pool,
            &self.shipyard_transaction.agent_symbol,
        )
        .await?;
        Ok(into_gql(agent))
    }

    async fn shipyard_ship(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLShipyardShip>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ShipyardShip::get_last_by_waypoint_and_ship_type(
            database_pool,
            &self.shipyard_transaction.waypoint_symbol,
            &self.shipyard_transaction.ship_type,
        )
        .await?;
        Ok(into_gql(erg))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ships = context.ship_manager.get_all_clone().await;
        let ship = ships
            .into_values()
            .find(|ship| ship.purchase_id == Some(self.shipyard_transaction.id));
        Ok(ship.map(|f| f.into()))
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
        self.survey
            .get_percent()
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
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg =
            database::Waypoint::get_by_symbol(database_pool, &self.survey.waypoint_symbol).await?;
        Ok(into_gql(erg))
    }

    async fn extractions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLExtraction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Extraction::get_by_survey_symbol(database_pool, &self.survey.signature)
            .await?;
        Ok(into_gql_vec(erg))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context.ship_manager.get_clone(&self.survey.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }

    async fn ship_state_before(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLShipState>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let ship_state =
            database::ShipState::get_by_id(database_pool, self.survey.ship_info_before).await?;
        Ok(into_gql(ship_state))
    }

    async fn ship_state_after(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLShipState>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let ship_state =
            database::ShipState::get_by_id(database_pool, self.survey.ship_info_after).await?;
        Ok(into_gql(ship_state))
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
    async fn waypoints(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Waypoint::get_by_system(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(erg))
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLMarketTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::MarketTransaction::get_by_system(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(transactions))
    }
    async fn shipyard_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLShipyardTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::ShipyardTransaction::get_by_system(database_pool, &self.system.symbol)
                .await?;
        Ok(into_gql_vec(transactions))
    }
    async fn chart_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLChartTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::ChartTransaction::get_by_system(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(transactions))
    }
    async fn repair_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLRepairTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::RepairTransaction::get_by_system(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(transactions))
    }
    async fn scrap_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLScrapTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::ScrapTransaction::get_by_system(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(transactions))
    }
    async fn ship_modification_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLShipModificationTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions = database::ShipModificationTransaction::get_by_system(
            database_pool,
            &self.system.symbol,
        )
        .await?;
        Ok(into_gql_vec(transactions))
    }

    async fn shipyard_ships(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLShipyardShip>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ShipyardShip::get_last_by_system(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(history))
    }

    async fn shipyard_ship_types(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLShipyardShipTypes>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ShipyardShipTypes::get_last_by_system(database_pool, &self.system.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn market_trades(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLMarketTrade>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::MarketTrade::get_last_by_system(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(history))
    }

    async fn market_trade_goods(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLMarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::MarketTradeGood::get_last_by_system(database_pool, &self.system.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn fleets(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLFleet>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::Fleet::get_by_system(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(history))
    }

    async fn surveys(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLSurvey>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::Survey::get_by_system_symbol(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(history))
    }

    async fn extractions(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLExtraction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::Extraction::get_by_system_symbol(database_pool, &self.system.symbol).await?;
        Ok(into_gql_vec(history))
    }

    async fn construction_materials(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLConstructionMaterial>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ConstructionMaterial::get_by_system(database_pool, &self.system.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn construction_shipments(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLConstructionShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ConstructionShipment::get_by_system(database_pool, &self.system.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn contract_deliveries(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLContractDelivery>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ContractDelivery::get_by_system_symbol(database_pool, &self.system.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    // async fn contract_shipments(
    //     &self,
    //     ctx: &async_graphql::Context<'_>,
    // ) -> Result<Vec<GQLContractShipment>> {
    //     // Changed return type
    //     let database_pool = ctx.data::<database::DbPool>().unwrap();
    //     let history =
    //         database::ContractShipment::get_by_system_symbol(database_pool, &self.system.symbol)
    //             .await?;
    //     Ok(into_gql_vec(history)) // Added conversion
    // }

    async fn ships(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ships_map = context.ship_manager.get_all_clone().await;
        Ok(ships_map
            .into_values()
            .filter(|ship| ship.nav.system_symbol == self.system.symbol)
            .map(|ship| ship.into())
            .collect())
    }

    async fn seen_agents(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<KnownAgent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system_market_transactions =
            database::MarketTransaction::get_by_system(database_pool, &self.system.symbol).await?;

        let system_shipyard_transactions =
            database::ShipyardTransaction::get_by_system(database_pool, &self.system.symbol)
                .await?;

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
    async fn agent(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLAgent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let agent = database::Agent::get_last_by_symbol(database_pool, &self.symbol).await?;
        Ok(into_gql(agent))
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
impl GQLTradeRoute {
    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.trade_route.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }

    async fn sell_waypoint(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.trade_route.sell_waypoint)
                .await?;
        Ok(into_gql(waypoint))
    }

    async fn purchase_waypoint(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint =
            database::Waypoint::get_by_symbol(database_pool, &self.trade_route.purchase_waypoint)
                .await?;
        Ok(into_gql(waypoint))
    }

    async fn reservation(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLReservedFund>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();

        let reservation = if let Some(reservation_id) = self.trade_route.reserved_fund {
            database::ReservedFund::get_by_id(database_pool, &reservation_id).await?
        } else {
            None
        };
        Ok(into_gql(reservation))
    }

    async fn transactions(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLMarketTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions =
            database::MarketTransaction::get_by_trade_route(database_pool, self.trade_route.id)
                .await?;
        Ok(into_gql_vec(transactions))
    }

    async fn market_transaction_summary(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<GQLTransactionSummary> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let summary = database::MarketTransaction::get_transaction_summary_by_trade_route(
            database_pool,
            self.trade_route.id,
        )
        .await?;
        Ok(summary.into())
    }

    async fn purchase_market_trade_good(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLMarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let market_trade_good = database::MarketTradeGood::get_by_last_waypoint_and_trade_symbol(
            database_pool,
            &self.trade_route.purchase_waypoint,
            &self.trade_route.symbol,
        )
        .await?;
        Ok(into_gql(market_trade_good))
    }

    async fn sell_market_trade_good(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLMarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let market_trade_good = database::MarketTradeGood::get_by_last_waypoint_and_trade_symbol(
            database_pool,
            &self.trade_route.sell_waypoint,
            &self.trade_route.symbol,
        )
        .await?;
        Ok(into_gql(market_trade_good))
    }
}

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
impl GQLWaypoint {
    async fn system(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system =
            database::System::get_by_symbol(database_pool, &self.waypoint.system_symbol).await?;
        Ok(into_gql(system))
    }

    async fn ships(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ships_map = context.ship_manager.get_all_clone().await;
        Ok(ships_map
            .into_values()
            .filter(|ship| ship.nav.waypoint_symbol == self.waypoint.symbol)
            .map(|ship| ship.into())
            .collect())
    }

    async fn chart_transaction<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLChartTransaction>> {
        // Changed return type to GQLChartTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ChartTransaction::get_by_waypoint_symbol(
            database_pool,
            &self.waypoint.symbol,
        )
        .await?;
        Ok(into_gql(reg)) // Added conversion
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLMarketTransaction>> {
        // Changed return type to GQLMarketTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::MarketTransaction::get_by_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn market_transaction_summary<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<GQLTransactionSummary> {
        // Changed return type to GQLTransactionSummary
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::MarketTransaction::get_transaction_summary_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
        )
        .await?;
        Ok(GQLTransactionSummary::from(reg)) // Added conversion
    }

    async fn shipyard_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLShipyardTransaction>> {
        // Changed return type to GQLShipyardTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::ShipyardTransaction::get_by_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn repair_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLRepairTransaction>> {
        // Changed return type to GQLRepairTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::RepairTransaction::get_by_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn scrap_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLScrapTransaction>> {
        // Changed return type to GQLScrapTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ScrapTransaction::get_by_waypoint(database_pool, &self.waypoint.symbol)
            .await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn ship_modification_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLShipModificationTransaction>> {
        // Changed return type to GQLShipModificationTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ShipModificationTransaction::get_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
        )
        .await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn construction_materials(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLConstructionMaterial>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ConstructionMaterial::get_by_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn construction_shipments_to(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLConstructionShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ConstructionShipment::get_by_destination_waypoint(
            database_pool,
            &self.waypoint.symbol,
        )
        .await?;
        Ok(into_gql_vec(history))
    }

    async fn construction_shipments_from(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLConstructionShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ConstructionShipment::get_by_source_waypoint(
            database_pool,
            &self.waypoint.symbol,
        )
        .await?;
        Ok(into_gql_vec(history))
    }

    async fn contract_deliveries(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLContractDelivery>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ContractDelivery::get_by_destination_symbol(
            database_pool,
            &self.waypoint.symbol,
        )
        .await?;
        Ok(into_gql_vec(history))
    }

    async fn contract_shipments_from(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLContractShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ContractShipment::get_by_source_symbol(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn contract_shipments_to(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLContractShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ContractShipment::get_by_destination_symbol(
            database_pool,
            &self.waypoint.symbol,
        )
        .await?;
        Ok(into_gql_vec(history))
    }

    async fn surveys(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLSurvey>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::Survey::get_by_waypoint_symbol(database_pool, &self.waypoint.symbol).await?;
        Ok(into_gql_vec(history))
    }

    async fn extractions(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLExtraction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::Extraction::get_by_waypoint_symbol(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn trade_routes_from(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLTradeRoute>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let routes =
            database::TradeRoute::get_by_purchase_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(routes))
    }

    async fn trade_routes_to(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLTradeRoute>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let routes =
            database::TradeRoute::get_by_sell_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(routes))
    }

    async fn market_trades(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLMarketTrade>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::MarketTrade::get_last_by_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn market_trade_goods(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLMarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::MarketTradeGood::get_last_by_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn shipyard_ships(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLShipyardShip>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ShipyardShip::get_last_by_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn shipyard_ship_types(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLShipyardShipTypes>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history =
            database::ShipyardShipTypes::get_last_by_waypoint(database_pool, &self.waypoint.symbol)
                .await?;
        Ok(into_gql_vec(history))
    }

    async fn shipyard<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShipyard>> {
        // Changed return type to GQLShipyard
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::Shipyard::get_last_by_waypoint(database_pool, &self.waypoint.symbol).await?;
        Ok(into_gql(reg)) // Added conversion
    }
}

#[async_graphql::ComplexObject]
impl crate::manager::budget_manager::BudgetInfo {
    async fn reservations(&self) -> Result<Vec<GQLReservedFund>> {
        Ok(into_gql_vec(self.reservations.clone()))
    }
}

#[async_graphql::ComplexObject]
impl crate::utils::RunInfo {
    async fn agent(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLAgent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let agent = database::Agent::get_last_by_symbol(database_pool, &self.agent_symbol).await?;
        Ok(into_gql(agent))
    }

    async fn headquarters_waypoint(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLWaypoint>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoint = database::Waypoint::get_by_symbol(database_pool, &self.headquarters).await?;
        Ok(into_gql(waypoint))
    }

    async fn headquarters_system(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system_symbol = utils::get_system_symbol(&self.headquarters);
        let system = database::System::get_by_symbol(database_pool, &system_symbol).await?;
        Ok(into_gql(system))
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "Ship")]
#[graphql(complex)]
pub struct GQLShip {
    #[graphql(flatten)]
    ship: ship::RustShip<ShipStatus>,
}

impl From<ship::RustShip<ShipStatus>> for GQLShip {
    fn from(value: ship::RustShip<ShipStatus>) -> Self {
        GQLShip { ship: value }
    }
}

#[async_graphql::ComplexObject]
impl GQLShip {
    async fn status(&self) -> Result<super::gql_ship::GQLShipStatus> {
        let status = self.ship.status.clone();
        Ok(status.into())
    }

    async fn purchase_transaction<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShipyardTransaction>> {
        // Changed return type to GQLShipyardTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if let Some(purchase_id) = self.ship.purchase_id {
            let erg = database::ShipyardTransaction::get_by_id(database_pool, purchase_id).await?;
            Ok(Some(erg.into())) // Added conversion
        } else {
            Ok(None)
        }
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLMarketTransaction>> {
        // Changed return type to GQLMarketTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::MarketTransaction::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn market_transaction_summary<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<GQLTransactionSummary> {
        // Changed return type to GQLTransactionSummary
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::MarketTransaction::get_transaction_summary_by_ship(
            database_pool,
            &self.ship.symbol,
        )
        .await?;
        Ok(GQLTransactionSummary::from(reg)) // Added conversion
    }

    async fn repair_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLRepairTransaction>> {
        // Changed return type to GQLRepairTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::RepairTransaction::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn scrap_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLScrapTransaction>> {
        // Changed return type to GQLScrapTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ScrapTransaction::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn ship_modification_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLShipModificationTransaction>> {
        // Changed return type to GQLShipModificationTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::ShipModificationTransaction::get_by_ship(database_pool, &self.ship.symbol)
                .await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn chart_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLChartTransaction>> {
        // Changed return type to GQLChartTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ChartTransaction::get_by_ship_symbol(database_pool, &self.ship.symbol)
            .await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn construction_shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLConstructionShipment>> {
        // Changed return type to GQLConstructionShipment
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg =
            database::ConstructionShipment::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn contract_shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLContractShipment>> {
        // Changed return type to GQLContractShipment
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ContractShipment::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn trade_routes<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLTradeRoute>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::TradeRoute::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg))
    }

    async fn surveys<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLSurvey>> {
        // Changed return type to GQLSurvey
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::Survey::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn extractions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLExtraction>> {
        // Changed return type to GQLExtraction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::Extraction::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn routes<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLRoute>> {
        // Changed return type to GQLRoute
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::Route::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn ship_jumps<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLShipJump>> {
        // Changed return type to GQLShipJump
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ShipJump::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn engine_info<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<GQLEngineInfo> {
        // Changed return type to GQLEngineInfo
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::EngineInfo::get_by_symbol(database_pool, &self.ship.engine).await?;
        Ok(GQLEngineInfo::from(reg)) // Added conversion
    }

    async fn frame_info<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<GQLFrameInfo> {
        // Changed return type to GQLFrameInfo
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::FrameInfo::get_by_symbol(database_pool, &self.ship.frame).await?;
        Ok(GQLFrameInfo::from(reg)) // Added conversion
    }

    async fn reactor_info<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<GQLReactorInfo> {
        // Changed return type to GQLReactorInfo
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ReactorInfo::get_by_symbol(database_pool, &self.ship.reactor).await?;
        Ok(GQLReactorInfo::from(reg)) // Added conversion
    }

    async fn ship_states<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<GQLShipState>> {
        // Changed return type to GQLShipState
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ShipState::get_by_ship(database_pool, &self.ship.symbol).await?;
        Ok(into_gql_vec(reg)) // Added conversion
    }

    async fn nav(&self) -> Result<GQLNavigationState> {
        Ok(GQLNavigationState::from(self.ship.nav.clone()))
    }

    async fn mounts(&self) -> Result<GQLMounts> {
        Ok(GQLMounts::from(self.ship.mounts.clone()))
    }

    async fn modules(&self) -> Result<GQLModules> {
        Ok(GQLModules::from(self.ship.modules.clone()))
    }
}
