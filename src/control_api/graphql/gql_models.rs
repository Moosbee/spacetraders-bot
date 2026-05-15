use std::collections::HashMap;

use async_graphql::dataloader::DataLoader;
use database::DatabaseConnectorAsync;
use serde::de::DeserializeOwned;
use ship::{
    status::ShipStatus, AutopilotState, CargoTradeCompletedEvent, JumpConnectionCompletedEvent,
    MiningExtractionCompletedEvent, MiningSurveyCreatedEvent, NavigateConnectionCompletedEvent,
    ShipEventPayload, WarpConnectionCompletedEvent,
};
use space_traders_client::models;

use crate::{
    control_api::graphql::gql_ship::{GQLModules, GQLMounts, GQLNavigationState, GQLShipStatus},
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

fn paginated_query(page: Option<i64>, page_size: Option<i64>) -> database::PaginatedQuery {
    database::PaginatedQuery::new(page.unwrap_or(1), page_size)
}

fn from_json_payload<T: DeserializeOwned>(
    payload: &sqlx::types::Json<serde_json::Value>,
    payload_name: &str,
) -> Result<T> {
    serde_json::from_value(payload.0.clone()).map_err(|err| {
        crate::error::Error::General(format!("failed to decode {payload_name}: {err}"))
    })
}

macro_rules! paginated_gql_object {
    ($name:ident, $graphql_name:literal, $db_ty:path, $gql_ty:ty) => {
        #[derive(Debug, Clone, async_graphql::SimpleObject)]
        #[graphql(name = $graphql_name)]
        pub struct $name {
            pub items: Vec<$gql_ty>,
            #[graphql(name = "totalCount")]
            pub total_count: i64,
            pub page: i64,
            #[graphql(name = "pageSize")]
            pub page_size: Option<i64>,
        }

        impl From<database::PaginatedResult<$db_ty>> for $name {
            fn from(value: database::PaginatedResult<$db_ty>) -> Self {
                Self {
                    items: into_gql_vec(value.items),
                    total_count: value.total_count,
                    page: value.page,
                    page_size: value.page_size,
                }
            }
        }
    };
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
        let reg = database::Agent::get_by_symbol(
            database_pool,
            &self.agent.symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?;
        Ok(into_gql_vec(reg.items))
    }

    async fn headquarters_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let reg = data_loader
            .load_one(self.agent.headquarters.clone())
            .await?;
        Ok(into_gql(reg))
    }

    async fn headquarters_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = utils::get_system_symbol(&self.agent.headquarters);
        let erg = database::System::get_by_id(database_pool, &system).await?;
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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let waypoint = data_loader
            .load_one(self.chart_transaction.waypoint_symbol.clone())
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

paginated_gql_object!(
    GQLChartTransactionPage,
    "ChartTransactionPage",
    database::ChartTransaction,
    GQLChartTransaction
);

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let waypoint = data_loader
            .load_one(self.construction_material.waypoint_symbol.clone())
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLConstructionShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ConstructionShipment::get_by_material_id(
            database_pool,
            self.construction_material.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
    }

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.construction_material.trade_symbol,
        }
    }
}

paginated_gql_object!(
    GQLConstructionMaterialPage,
    "ConstructionMaterialPage",
    database::ConstructionMaterial,
    GQLConstructionMaterial
);

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let waypoint = data_loader
            .load_one(
                self.construction_shipment
                    .construction_site_waypoint
                    .clone(),
            )
            .await?;
        Ok(into_gql(waypoint))
    }

    async fn purchase_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let waypoint = data_loader
            .load_one(self.construction_shipment.purchase_waypoint.clone())
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
            &self.construction_shipment.material_id,
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTransaction::get_by_construction(
            database_pool,
            self.construction_shipment.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
    }

    async fn purchase_market_trade_good(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLMarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let market_trade_good = database::MarketTradeGood::get_by_last_waypoint_and_trade_symbol(
            database_pool,
            &self.construction_shipment.purchase_waypoint,
            &self.construction_shipment.trade_symbol,
        )
        .await?;
        Ok(into_gql(market_trade_good))
    }

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.construction_shipment.trade_symbol,
        }
    }
}

paginated_gql_object!(
    GQLConstructionShipmentPage,
    "ConstructionShipmentPage",
    database::ConstructionShipment,
    GQLConstructionShipment
);

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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLContractDeliveryPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ContractDelivery::get_by_contract_id(
            database_pool,
            &self.contract.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
    }

    async fn shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLContractShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ContractShipment::get_by_contract_id(
            database_pool,
            &self.contract.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTransaction::get_by_contract(
            database_pool,
            &self.contract.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
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

paginated_gql_object!(
    GQLContractPage,
    "ContractPage",
    database::Contract,
    GQLContract
);

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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLContractShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ContractShipment::get_by_contract_id_trade_symbol_destination_symbol(
            database_pool,
            &self.contract_delivery.contract_id,
            &self.contract_delivery.trade_symbol,
            &self.contract_delivery.destination_symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
    }

    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.contract_delivery.destination_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.contract_delivery.trade_symbol,
        }
    }
}

paginated_gql_object!(
    GQLContractDeliveryPage,
    "ContractDeliveryPage",
    database::ContractDelivery,
    GQLContractDelivery
);

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.contract_shipment.destination_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn purchase_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.contract_shipment.purchase_symbol.clone())
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

    async fn purchase_market_trade_good(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLMarketTradeGood>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let market_trade_good = database::MarketTradeGood::get_by_last_waypoint_and_trade_symbol(
            database_pool,
            &self.contract_shipment.purchase_symbol,
            &self.contract_shipment.trade_symbol,
        )
        .await?;
        Ok(into_gql(market_trade_good))
    }
    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.contract_shipment.trade_symbol,
        }
    }
}

paginated_gql_object!(
    GQLContractShipmentPage,
    "ContractShipmentPage",
    database::ContractShipment,
    GQLContractShipment
);

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
impl GQLEngineInfo {
    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.engine_info.symbol.into(),
        }
    }
}

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.extraction.waypoint_symbol.clone())
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
    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.extraction.yield_symbol,
        }
    }
}

paginated_gql_object!(
    GQLExtractionPage,
    "ExtractionPage",
    database::Extraction,
    GQLExtraction
);

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

paginated_gql_object!(GQLFleetPage, "FleetPage", database::Fleet, GQLFleet);

#[async_graphql::ComplexObject]
impl GQLFleet {
    async fn config(&self) -> Result<database::FleetConfig> {
        let erg: database::FleetConfig = self.fleet.get_config()?;
        Ok(erg)
    }

    async fn system<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::System::get_by_id(database_pool, &self.fleet.system_symbol).await?;
        Ok(into_gql(erg))
    }

    async fn assignments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipAssignmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::ShipAssignment::get_by_fleet_id(
            database_pool,
            self.fleet.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
    }

    async fn all_ships<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLShip>> {
        // let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        // let all_ships = context.ship_manager.get_all_clone().await;
        let ship_loader = ctx.data::<DataLoader<super::AllShipLoader>>().unwrap();
        let all_ships = ship_loader.load_one(()).await?.unwrap();
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
        let ship_loader = ctx.data::<DataLoader<super::AllShipLoader>>().unwrap();
        let all_ships = ship_loader.load_one(()).await?.unwrap();
        let ships = all_ships
            .into_values()
            .filter(|ship| ship.status.fleet_id == Some(self.fleet.id))
            .collect::<Vec<_>>();
        Ok(ships.into_iter().map(|s| s.into()).collect())
    }
    async fn temp_ships<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLShip>> {
        let ship_loader = ctx.data::<DataLoader<super::AllShipLoader>>().unwrap();
        let all_ships = ship_loader.load_one(()).await?.unwrap();
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
impl GQLFrameInfo {
    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.frame_info.symbol.into(),
        }
    }
}

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

paginated_gql_object!(
    GQLJumpGateConnectionPage,
    "JumpGateConnectionPage",
    database::JumpGateConnection,
    GQLJumpGateConnection
);
#[async_graphql::ComplexObject]
impl GQLJumpGateConnection {
    async fn waypoint_from<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.jump_gate_connection.from.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn waypoint_to<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.jump_gate_connection.to.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn system_from<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = utils::get_system_symbol(&self.jump_gate_connection.from);
        let erg = database::System::get_by_id(database_pool, &system).await?;
        Ok(into_gql(erg))
    }

    async fn system_to<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = utils::get_system_symbol(&self.jump_gate_connection.to);
        let erg = database::System::get_by_id(database_pool, &system).await?;
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

paginated_gql_object!(
    GQLMarketTradePage,
    "MarketTradePage",
    database::MarketTrade,
    GQLMarketTrade
);

#[async_graphql::ComplexObject]
impl GQLMarketTrade {
    async fn waypoint(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.market_trade.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTradePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTrade::get_history_by_waypoint_and_trade_symbol(
            database_pool,
            &self.market_trade.waypoint_symbol,
            &self.market_trade.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
    }

    async fn maps(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTradePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let maping = match self.market_trade.r#type {
            models::market_trade_good::Type::Export => {
                database::ExportImportMapping::get_imports_for_export(
                    database_pool,
                    self.market_trade.symbol,
                    database::PaginatedQuery::unpaged(),
                )
                .await?
                .items
            }
            models::market_trade_good::Type::Exchange => Vec::new(),
            models::market_trade_good::Type::Import => {
                database::ExportImportMapping::get_exports_for_import(
                    database_pool,
                    self.market_trade.symbol,
                    database::PaginatedQuery::unpaged(),
                )
                .await?
                .items
            }
        };

        let market_trades = database::MarketTrade::get_last_by_waypoint(
            database_pool,
            &self.market_trade.waypoint_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        Ok(database::paginate_items(
            paginated_query(page, page_size),
            market_trades
                .into_iter()
                .filter(|t| maping.contains(&t.symbol))
                .collect(),
        )?
        .into())
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

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.market_trade.symbol,
        }
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

paginated_gql_object!(
    GQLMarketTradeGoodPage,
    "MarketTradeGoodPage",
    database::MarketTradeGood,
    GQLMarketTradeGood
);

#[async_graphql::ComplexObject]
impl GQLMarketTradeGood {
    async fn waypoint(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.market_trade_good.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTradeGoodPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::MarketTradeGood::get_history_by_waypoint_and_trade_symbol(
            database_pool,
            &self.market_trade_good.waypoint_symbol,
            &self.market_trade_good.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTransaction::get_by_waypoint_and_trade_symbol(
            database_pool,
            &self.market_trade_good.waypoint_symbol,
            self.market_trade_good.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
    }

    async fn market_trade(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLMarketTrade>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::MarketTrade::get_last_by_waypoint_and_trade_symbol(
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
    async fn maps(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTradeGoodPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let maping = match self.market_trade_good.r#type {
            models::market_trade_good::Type::Export => {
                database::ExportImportMapping::get_imports_for_export(
                    database_pool,
                    self.market_trade_good.symbol,
                    database::PaginatedQuery::unpaged(),
                )
                .await?
                .items
            }
            models::market_trade_good::Type::Exchange => Vec::new(),
            models::market_trade_good::Type::Import => {
                database::ExportImportMapping::get_exports_for_import(
                    database_pool,
                    self.market_trade_good.symbol,
                    database::PaginatedQuery::unpaged(),
                )
                .await?
                .items
            }
        };

        let market_trades = database::MarketTradeGood::get_last_by_waypoint(
            database_pool,
            &self.market_trade_good.waypoint_symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        Ok(database::paginate_items(
            paginated_query(page, page_size),
            market_trades
                .into_iter()
                .filter(|t| maping.contains(&t.symbol))
                .collect(),
        )?
        .into())
    }

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.market_trade_good.symbol,
        }
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

paginated_gql_object!(
    GQLMarketTransactionPage,
    "MarketTransactionPage",
    database::MarketTransaction,
    GQLMarketTransaction
);
#[async_graphql::ComplexObject]
impl GQLMarketTransaction {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.market_transaction.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
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
            database::TradeRoute::get_by_id(database_pool, &trade_route).await?
        } else {
            None
        };
        Ok(into_gql(trade_route))
    }

    pub async fn mining_waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = if let Some(waypoint) = self.market_transaction.mining.clone() {
            data_loader.load_one(waypoint).await?
        } else {
            None
        };
        Ok(into_gql(erg))
    }

    pub async fn construction_shipment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLConstructionShipment>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let construction_shipment = if let Some(construction_shipment) =
            self.market_transaction.construction
        {
            database::ConstructionShipment::get_by_id(database_pool, &construction_shipment).await?
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

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.market_transaction.trade_symbol,
        }
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
impl GQLModuleInfo {
    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.module_info.symbol.into(),
        }
    }
}

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
impl GQLMountInfo {
    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.mount_info.symbol.into(),
        }
    }
}

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
impl GQLReactorInfo {
    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.reactor_info.symbol.into(),
        }
    }
}

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.repair_transaction.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.repair_transaction.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }
}

paginated_gql_object!(
    GQLRepairTransactionPage,
    "RepairTransactionPage",
    database::RepairTransaction,
    GQLRepairTransaction
);

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
    async fn contract<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLContractPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let contract = database::Contract::get_by_reservation_id(
            database_pool,
            self.reserved_fund.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(contract.into())
    }

    async fn trade_route<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLTradeRoutePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let trade_routes = database::TradeRoute::get_by_reservation_id(
            database_pool,
            self.reserved_fund.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(trade_routes.into())
    }

    async fn construction_shipment<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLConstructionShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let construction_shipments = database::ConstructionShipment::get_by_reservation_id(
            database_pool,
            self.reserved_fund.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(construction_shipments.into())
    }
}

paginated_gql_object!(
    GQLReservedFundPage,
    "ReservedFundPage",
    database::ReservedFund,
    GQLReservedFund
);

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "JumpConnectionCompletedEvent")]
pub struct GQLJumpConnectionCompletedEvent {
    pub from: String,
    pub to: String,
    pub distance: i64,
}

impl From<JumpConnectionCompletedEvent> for GQLJumpConnectionCompletedEvent {
    fn from(value: JumpConnectionCompletedEvent) -> Self {
        Self {
            from: value.from,
            to: value.to,
            distance: value.distance,
        }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "WarpConnectionCompletedEvent")]
pub struct GQLWarpConnectionCompletedEvent {
    pub from: String,
    pub to: String,
    pub nav_mode: String,
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: f64,
}

impl From<WarpConnectionCompletedEvent> for GQLWarpConnectionCompletedEvent {
    fn from(value: WarpConnectionCompletedEvent) -> Self {
        Self {
            from: value.from,
            to: value.to,
            nav_mode: value.nav_mode,
            distance: value.distance,
            fuel_cost: value.fuel_cost,
            travel_time: value.travel_time,
        }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "NavigateConnectionCompletedEvent")]
pub struct GQLNavigateConnectionCompletedEvent {
    pub from: String,
    pub to: String,
    pub nav_mode: String,
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: f64,
}

impl From<NavigateConnectionCompletedEvent> for GQLNavigateConnectionCompletedEvent {
    fn from(value: NavigateConnectionCompletedEvent) -> Self {
        Self {
            from: value.from,
            to: value.to,
            nav_mode: value.nav_mode,
            distance: value.distance,
            fuel_cost: value.fuel_cost,
            travel_time: value.travel_time,
        }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "CargoTradeCompletedEvent")]
pub struct GQLCargoTradeCompletedEvent {
    pub waypoint_symbol: String,
    pub trade_symbol: models::TradeSymbol,
    pub transaction_type: models::market_transaction::Type,
    pub units: i32,
    pub price_per_unit: i32,
    pub total_price: i32,
    pub contract_id: Option<String>,
    pub trade_route_id: Option<i32>,
    pub mining_waypoint_symbol: Option<String>,
    pub construction_shipment_id: Option<i64>,
}

impl From<CargoTradeCompletedEvent> for GQLCargoTradeCompletedEvent {
    fn from(value: CargoTradeCompletedEvent) -> Self {
        Self {
            waypoint_symbol: value.waypoint_symbol,
            trade_symbol: value.trade_symbol,
            transaction_type: value.transaction_type,
            units: value.units,
            price_per_unit: value.price_per_unit,
            total_price: value.total_price,
            contract_id: value.contract_id,
            trade_route_id: value.trade_route_id,
            mining_waypoint_symbol: value.mining_waypoint_symbol,
            construction_shipment_id: value.construction_shipment_id,
        }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MiningExtractionCompletedEvent")]
pub struct GQLMiningExtractionCompletedEvent {
    pub waypoint_symbol: String,
    pub siphon: bool,
    pub yield_symbol: models::TradeSymbol,
    pub yield_units: i32,
    pub survey_signature: Option<String>,
}

impl From<MiningExtractionCompletedEvent> for GQLMiningExtractionCompletedEvent {
    fn from(value: MiningExtractionCompletedEvent) -> Self {
        Self {
            waypoint_symbol: value.waypoint_symbol,
            siphon: value.siphon,
            yield_symbol: value.yield_symbol,
            yield_units: value.yield_units,
            survey_signature: value.survey_signature,
        }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "MiningSurveyCreatedEvent")]
pub struct GQLMiningSurveyCreatedEvent {
    pub waypoint_symbol: String,
    pub surveys_created: i32,
    pub survey_signatures: Vec<String>,
}

impl From<MiningSurveyCreatedEvent> for GQLMiningSurveyCreatedEvent {
    fn from(value: MiningSurveyCreatedEvent) -> Self {
        Self {
            waypoint_symbol: value.waypoint_symbol,
            surveys_created: value.surveys_created,
            survey_signatures: value.survey_signatures,
        }
    }
}

#[derive(Debug, Clone, async_graphql::Union)]
#[graphql(name = "ShipEventPayload")]
pub enum GQLShipEventPayload {
    JumpConnectionCompleted(GQLJumpConnectionCompletedEvent),
    WarpConnectionCompleted(GQLWarpConnectionCompletedEvent),
    NavigateConnectionCompleted(GQLNavigateConnectionCompletedEvent),
    CargoTradeCompleted(GQLCargoTradeCompletedEvent),
    MiningExtractionCompleted(GQLMiningExtractionCompletedEvent),
    MiningSurveyCreated(GQLMiningSurveyCreatedEvent),
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "ShipEvent")]
#[graphql(complex)]
pub struct GQLShipEvent {
    #[graphql(flatten)]
    ship_event: database::ShipEvent,
}

impl From<database::ShipEvent> for GQLShipEvent {
    fn from(value: database::ShipEvent) -> Self {
        GQLShipEvent { ship_event: value }
    }
}

#[async_graphql::ComplexObject]
impl GQLShipEvent {
    async fn payload(&self) -> Result<GQLShipEventPayload> {
        let payload =
            from_json_payload::<ShipEventPayload>(&self.ship_event.payload, "ship_event.payload")?;
        Ok(match payload {
            ShipEventPayload::JumpConnectionCompleted(payload) => {
                GQLShipEventPayload::JumpConnectionCompleted(payload.into())
            }
            ShipEventPayload::WarpConnectionCompleted(payload) => {
                GQLShipEventPayload::WarpConnectionCompleted(payload.into())
            }
            ShipEventPayload::NavigateConnectionCompleted(payload) => {
                GQLShipEventPayload::NavigateConnectionCompleted(payload.into())
            }
            ShipEventPayload::CargoTradeCompleted(payload) => {
                GQLShipEventPayload::CargoTradeCompleted(payload.into())
            }
            ShipEventPayload::MiningExtractionCompleted(payload) => {
                GQLShipEventPayload::MiningExtractionCompleted(payload.into())
            }
            ShipEventPayload::MiningSurveyCreated(payload) => {
                GQLShipEventPayload::MiningSurveyCreated(payload.into())
            }
        })
    }

    async fn payload_json(&self) -> Result<String> {
        serde_json::to_string(&self.ship_event.payload.0).map_err(|err| {
            crate::error::Error::General(format!("failed to encode ship_event.payload_json: {err}"))
        })
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context.ship_manager.get_clone(&self.ship_event.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }

    async fn before_ship_state<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShipState>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let ship_state =
            database::ShipState::get_by_id(database_pool, &self.ship_event.before_ship_state_id)
                .await?;
        Ok(into_gql(ship_state))
    }

    async fn after_ship_state<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShipState>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let ship_state =
            database::ShipState::get_by_id(database_pool, &self.ship_event.after_ship_state_id)
                .await?;
        Ok(into_gql(ship_state))
    }
}

paginated_gql_object!(
    GQLShipEventPage,
    "ShipEventPage",
    database::ShipEvent,
    GQLShipEvent
);

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.scrap_transaction.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.scrap_transaction.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }
}

paginated_gql_object!(
    GQLScrapTransactionPage,
    "ScrapTransactionPage",
    database::ScrapTransaction,
    GQLScrapTransaction
);

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

paginated_gql_object!(
    GQLShipAssignmentPage,
    "ShipAssignmentPage",
    database::ShipAssignment,
    GQLShipAssignment
);

#[async_graphql::ComplexObject]
impl GQLShipAssignment {
    async fn fleet<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLFleet>> {
        // let database_pool = ctx.data::<database::DbPool>().unwrap();
        // let erg = database::Fleet::get_by_id(database_pool, self.ship_assignment.fleet_id).await?;
        let fleet_loader = ctx.data::<DataLoader<database::FleetLoader>>().unwrap();
        let erg = fleet_loader.load_one(self.ship_assignment.fleet_id).await?;
        Ok(into_gql(erg))
    }

    async fn ship<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLShip>> {
        let ship_loader = ctx.data::<DataLoader<super::AllShipLoader>>().unwrap();
        let all_ships = ship_loader.load_one(()).await?.unwrap();
        let ship = all_ships.into_values().find(|ship| {
            ship.status.assignment_id == Some(self.ship_assignment.id)
                || ship.status.temp_assignment_id == Some(self.ship_assignment.id)
        });
        Ok(ship.map(|f| f.into()))
    }

    async fn permanent_ship<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLShip>> {
        let ship_loader = ctx.data::<DataLoader<super::AllShipLoader>>().unwrap();
        let all_ships = ship_loader.load_one(()).await?.unwrap();
        let ship = all_ships
            .into_values()
            .find(|ship| ship.status.assignment_id == Some(self.ship_assignment.id));
        Ok(ship.map(|f| f.into()))
    }

    async fn temp_ship<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Option<GQLShip>> {
        let ship_loader = ctx.data::<DataLoader<super::AllShipLoader>>().unwrap();
        let all_ships = ship_loader.load_one(()).await?.unwrap();
        let ship = all_ships
            .into_values()
            .find(|ship| ship.status.temp_assignment_id == Some(self.ship_assignment.id));
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

paginated_gql_object!(
    GQLShipInfoPage,
    "ShipInfoPage",
    database::ShipInfo,
    GQLShipInfo
);

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.ship_modification_transaction.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context
            .ship_manager
            .get_clone(&self.ship_modification_transaction.ship_symbol);
        Ok(ship.map(|f| f.into()))
    }

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.ship_modification_transaction.trade_symbol,
        }
    }
}

paginated_gql_object!(
    GQLShipModificationTransactionPage,
    "ShipModificationTransactionPage",
    database::ShipModificationTransaction,
    GQLShipModificationTransaction
);

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

paginated_gql_object!(
    GQLShipStatePage,
    "ShipStatePage",
    database::ShipState,
    GQLShipState
);

#[async_graphql::ComplexObject]
impl GQLShipState {
    async fn cargo_inventory(&self) -> HashMap<models::TradeSymbol, i32> {
        self.ship_state.cargo_inventory.0.clone()
    }

    async fn status(&self) -> Result<GQLShipStatus> {
        let status: ShipStatus = from_json_payload(&self.ship_state.status, "ship_state.status")?;
        Ok(status.into())
    }

    async fn auto_pilot_state(&self) -> Result<Option<AutopilotState>> {
        from_json_payload(
            &self.ship_state.auto_pilot_state,
            "ship_state.auto_pilot_state",
        )
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
            database::System::get_by_id(database_pool, &self.ship_state.system_symbol).await?;
        Ok(into_gql(system))
    }

    async fn waypoint_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.ship_state.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn route_destination_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.ship_state.route_destination_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn route_destination_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system =
            database::System::get_by_id(database_pool, &self.ship_state.route_destination_system)
                .await?;
        Ok(into_gql(system))
    }

    async fn route_origin_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.ship_state.route_origin_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn route_origin_system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system =
            database::System::get_by_id(database_pool, &self.ship_state.route_origin_system)
                .await?;
        Ok(into_gql(system))
    }

    async fn auto_pilot_destination_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = if let Some(ap_dest) = &self.ship_state.auto_pilot_destination_symbol {
            data_loader.load_one(ap_dest.clone()).await?
        } else {
            None
        };
        Ok(into_gql(erg))
    }

    async fn auto_pilot_destination_system_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = if let Some(ap_dest) = &self.ship_state.auto_pilot_destination_system_symbol {
            database::System::get_by_id(database_pool, ap_dest).await?
        } else {
            None
        };
        Ok(into_gql(system))
    }

    async fn auto_pilot_origin_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = if let Some(ap_orig) = &self.ship_state.auto_pilot_origin_symbol {
            data_loader.load_one(ap_orig.clone()).await?
        } else {
            None
        };
        Ok(into_gql(erg))
    }

    async fn auto_pilot_origin_system_symbol<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system = if let Some(ap_orig) = &self.ship_state.auto_pilot_origin_system_symbol {
            database::System::get_by_id(database_pool, ap_orig).await?
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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.shipyard.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::Shipyard::get_history_by_waypoint(
            database_pool,
            &self.shipyard.waypoint_symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn shipyard_ship_types(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardShipTypesPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShipTypes::get_last_by_waypoint(
            database_pool,
            &self.shipyard.waypoint_symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn shipyard_ships(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardShipPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShip::get_last_by_waypoint(
            database_pool,
            &self.shipyard.waypoint_symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn shipyard_transactions(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardTransaction::get_by_waypoint(
            database_pool,
            &self.shipyard.waypoint_symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.shipyard_ship.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn history(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardShipPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShip::get_history_by_waypoint_and_ship_type(
            database_pool,
            &self.shipyard_ship.waypoint_symbol,
            &self.shipyard_ship.ship_type,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn shipyard_transactions(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardTransaction::get_by_waypoint_and_ship_type(
            database_pool,
            &self.shipyard_ship.waypoint_symbol,
            self.shipyard_ship.ship_type,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.shipyard_ship.ship_type.into(),
        }
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
            database::Shipyard::get_by_id(database_pool, &self.shipyard_ship_types.shipyard_id)
                .await?;
        Ok(into_gql(erg))
    }

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.shipyard_ship_types.ship_type.into(),
        }
    }
}

paginated_gql_object!(
    GQLShipyardPage,
    "ShipyardPage",
    database::Shipyard,
    GQLShipyard
);
paginated_gql_object!(
    GQLShipyardShipPage,
    "ShipyardShipPage",
    database::ShipyardShip,
    GQLShipyardShip
);
paginated_gql_object!(
    GQLShipyardShipTypesPage,
    "ShipyardShipTypesPage",
    database::ShipyardShipTypes,
    GQLShipyardShipTypes
);

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.shipyard_transaction.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
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
        let ship_loader = ctx.data::<DataLoader<super::AllShipLoader>>().unwrap();
        let all_ships = ship_loader.load_one(()).await?.unwrap();
        let ship = all_ships
            .into_values()
            .find(|ship| ship.purchase_id == Some(self.shipyard_transaction.id));
        Ok(ship.map(|f| f.into()))
    }

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.shipyard_transaction.ship_type.into(),
        }
    }
}

paginated_gql_object!(
    GQLShipyardTransactionPage,
    "ShipyardTransactionPage",
    database::ShipyardTransaction,
    GQLShipyardTransaction
);

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.survey.waypoint_symbol.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn extractions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLExtractionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let erg = database::Extraction::get_by_survey_symbol(
            database_pool,
            &self.survey.signature,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(erg.into())
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
            database::ShipState::get_by_id(database_pool, &self.survey.ship_info_before).await?;
        Ok(into_gql(ship_state))
    }

    async fn ship_state_after(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLShipState>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let ship_state =
            database::ShipState::get_by_id(database_pool, &self.survey.ship_info_after).await?;
        Ok(into_gql(ship_state))
    }
}

paginated_gql_object!(GQLSurveyPage, "SurveyPage", database::Survey, GQLSurvey);

#[derive(Debug, Clone, serde::Serialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct SurveyPercent {
    pub symbol: models::TradeSymbol,
    pub percent: f64,
}

#[async_graphql::ComplexObject]
impl SurveyPercent {
    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.symbol,
        }
    }
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
    async fn waypoints(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLWaypointPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let waypoints = database::Waypoint::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(waypoints.into())
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions = database::MarketTransaction::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(transactions.into())
    }
    async fn shipyard_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions = database::ShipyardTransaction::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(transactions.into())
    }
    async fn chart_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLChartTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions = database::ChartTransaction::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(transactions.into())
    }
    async fn repair_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLRepairTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions = database::RepairTransaction::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(transactions.into())
    }
    async fn scrap_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLScrapTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions = database::ScrapTransaction::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(transactions.into())
    }
    async fn ship_modification_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipModificationTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions = database::ShipModificationTransaction::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(transactions.into())
    }

    async fn shipyard_ships(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardShipPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShip::get_last_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn shipyard_ship_types(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardShipTypesPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShipTypes::get_last_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn market_trades(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTradePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::MarketTrade::get_last_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn market_trade_goods(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTradeGoodPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::MarketTradeGood::get_last_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn fleets(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLFleetPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let fleets = database::Fleet::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(fleets.into())
    }

    async fn surveys(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLSurveyPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::Survey::get_by_system_symbol(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn extractions(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLExtractionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::Extraction::get_by_system_symbol(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn construction_materials(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLConstructionMaterialPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ConstructionMaterial::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn construction_shipments(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLConstructionShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ConstructionShipment::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn contract_deliveries(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLContractDeliveryPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ContractDelivery::get_by_system_symbol(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn jump_gate_connections(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLJumpGateConnectionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let jump_gates = database::JumpGateConnection::get_all_from_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;

        Ok(jump_gates.into())
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

    async fn trade_routes(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLTradeRoutePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let routes = database::TradeRoute::get_by_system(
            database_pool,
            &self.system.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(routes.into())
    }

    async fn ships(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLShip>> {
        let ship_loader = ctx.data::<DataLoader<super::AllShipLoader>>().unwrap();
        let all_ships = ship_loader.load_one(()).await?.unwrap();
        Ok(all_ships
            .into_values()
            .filter(|ship| ship.nav.system_symbol == self.system.symbol)
            .map(|ship| ship.into())
            .collect())
    }

    async fn seen_agents(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<KnownAgent>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system_market_transactions = database::MarketTransaction::get_by_system(
            database_pool,
            &self.system.symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

        let system_shipyard_transactions = database::ShipyardTransaction::get_by_system(
            database_pool,
            &self.system.symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

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

paginated_gql_object!(GQLSystemPage, "SystemPage", database::System, GQLSystem);

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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.trade_route.sell_waypoint.clone())
            .await?;
        Ok(into_gql(erg))
    }

    async fn purchase_waypoint(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader
            .load_one(self.trade_route.purchase_waypoint.clone())
            .await?;
        Ok(into_gql(erg))
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let transactions = database::MarketTransaction::get_by_trade_route(
            database_pool,
            self.trade_route.id,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(transactions.into())
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

    async fn trade_symbol_info(&self) -> TradeSymbolInfo {
        TradeSymbolInfo {
            symbol: self.trade_route.symbol,
        }
    }
}

paginated_gql_object!(
    GQLTradeRoutePage,
    "TradeRoutePage",
    database::TradeRoute,
    GQLTradeRoute
);

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

paginated_gql_object!(
    GQLWaypointPage,
    "WaypointPage",
    database::Waypoint,
    GQLWaypoint
);

#[async_graphql::ComplexObject]
impl GQLWaypoint {
    async fn system(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system =
            database::System::get_by_id(database_pool, &self.waypoint.system_symbol).await?;
        Ok(into_gql(system))
    }

    async fn ships(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<GQLShip>> {
        let ship_loader = ctx.data::<DataLoader<super::AllShipLoader>>().unwrap();
        let all_ships = ship_loader.load_one(()).await?.unwrap();
        Ok(all_ships
            .into_values()
            .filter(|ship| ship.nav.waypoint_symbol == self.waypoint.symbol)
            .map(|ship| ship.into())
            .collect())
    }

    async fn chart_transaction<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLChartTransaction>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ChartTransaction::get_by_waypoint_symbol(
            database_pool,
            &self.waypoint.symbol,
        )
        .await?;
        Ok(into_gql(reg))
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::MarketTransaction::get_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into())
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardTransactionPage> {
        // Changed return type to GQLShipyardTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ShipyardTransaction::get_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into()) // Added conversion
    }

    async fn repair_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLRepairTransactionPage> {
        // Changed return type to GQLRepairTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::RepairTransaction::get_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into()) // Added conversion
    }

    async fn scrap_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLScrapTransactionPage> {
        // Changed return type to GQLScrapTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ScrapTransaction::get_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into()) // Added conversion
    }

    async fn ship_modification_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipModificationTransactionPage> {
        // Changed return type to GQLShipModificationTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ShipModificationTransaction::get_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into()) // Added conversion
    }

    async fn construction_materials(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLConstructionMaterialPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ConstructionMaterial::get_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn construction_shipments_to(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLConstructionShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ConstructionShipment::get_by_destination_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn construction_shipments_from(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLConstructionShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ConstructionShipment::get_by_source_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn contract_deliveries(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLContractDeliveryPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ContractDelivery::get_by_destination_symbol(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn contract_shipments_from(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLContractShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ContractShipment::get_by_source_symbol(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn contract_shipments_to(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLContractShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ContractShipment::get_by_destination_symbol(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn surveys(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLSurveyPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::Survey::get_by_waypoint_symbol(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn extractions(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLExtractionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::Extraction::get_by_waypoint_symbol(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn trade_routes_from(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLTradeRoutePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let routes = database::TradeRoute::get_by_purchase_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(routes.into())
    }

    async fn trade_routes_to(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLTradeRoutePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let routes = database::TradeRoute::get_by_sell_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(routes.into())
    }

    async fn trade_routes(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLTradeRoutePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let routes = database::TradeRoute::get_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(routes.into())
    }

    async fn market_trades(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTradePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::MarketTrade::get_last_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn market_trade_goods(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTradeGoodPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::MarketTradeGood::get_last_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn shipyard_ships(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardShipPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShip::get_last_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
    }

    async fn shipyard_ship_types(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipyardShipTypesPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let history = database::ShipyardShipTypes::get_last_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(history.into())
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

    async fn jump_gate_connections(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLJumpGateConnectionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let connections = database::JumpGateConnection::get_all_from(
            database_pool,
            &self.waypoint.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(connections.into())
    }

    async fn last_scrap(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        if !(self.waypoint.has_marketplace || self.waypoint.has_shipyard) {
            return Ok(None);
        }
        let last_scrap = database::MarketTradeGood::get_last_by_waypoint(
            database_pool,
            &self.waypoint.symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items
        .iter()
        .max_by(|a, b| a.created_at.cmp(&b.created_at))
        .map(|f| f.created_at)
        .unwrap_or(chrono::DateTime::<chrono::Utc>::MIN_UTC);

        Ok(Some(last_scrap))
    }

    async fn next_scrap(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        if !(self.waypoint.has_marketplace || self.waypoint.has_shipyard) {
            return Ok(None);
        }
        let market_trade_goods = database::MarketTradeGood::get_last_by_waypoint(
            &context.database_pool,
            &self.waypoint.symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;
        let max_update_interval = { context.config.read().await.max_update_interval };

        let next_time = crate::manager::scrapping_manager::priority_calculator::get_waypoint_time(
            market_trade_goods
                .into_iter()
                .map(From::from)
                .collect::<Vec<_>>()
                .as_slice(),
            max_update_interval,
        )?;

        Ok(Some(next_time))
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
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.headquarters.clone()).await?;
        Ok(into_gql(erg))
    }

    async fn headquarters_system(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Option<GQLSystem>> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let system_symbol = utils::get_system_symbol(&self.headquarters);
        let system = database::System::get_by_id(database_pool, &system_symbol).await?;
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

    async fn possible_scraps(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<ScrapInfo>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();

        let waypoints = context
            .scrapping_manager
            .get_info(self.ship.clone())
            .await?;
        Ok(waypoints
            .into_iter()
            .map(|f| ScrapInfo {
                waypoint_symbol: f.0.clone(),
                date: f.1,
            })
            .collect())
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLMarketTransactionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::MarketTransaction::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into())
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLRepairTransactionPage> {
        // Changed return type to GQLRepairTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::RepairTransaction::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into()) // Added conversion
    }

    async fn scrap_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLScrapTransactionPage> {
        // Changed return type to GQLScrapTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ScrapTransaction::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into()) // Added conversion
    }

    async fn ship_modification_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipModificationTransactionPage> {
        // Changed return type to GQLShipModificationTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ShipModificationTransaction::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into()) // Added conversion
    }

    async fn chart_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLChartTransactionPage> {
        // Changed return type to GQLChartTransaction
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ChartTransaction::get_by_ship_symbol(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into()) // Added conversion
    }

    async fn construction_shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLConstructionShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ConstructionShipment::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into())
    }

    async fn contract_shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLContractShipmentPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ContractShipment::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into())
    }

    async fn trade_routes<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLTradeRoutePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::TradeRoute::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into())
    }

    async fn surveys<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLSurveyPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::Survey::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into())
    }

    async fn extractions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLExtractionPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::Extraction::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into())
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipStatePage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ShipState::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into())
    }

    async fn ship_events<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLShipEventPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::ShipEvent::get_by_ship(
            database_pool,
            &self.ship.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reg.into())
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

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(name = "GateConn")]
#[graphql(complex)]
pub struct GateConn {
    pub under_construction_a: bool,
    pub under_construction_b: bool,
    pub point_a_symbol: String,
    pub point_b_symbol: String,
    pub from_a: bool,
    pub from_b: bool,
}

paginated_gql_object!(GQLGateConnPage, "GateConnPage", GateConn, GateConn);

#[async_graphql::ComplexObject]
impl GateConn {
    async fn point_a(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.point_a_symbol.clone()).await?;
        Ok(into_gql(erg))
    }

    async fn point_b(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.point_b_symbol.clone()).await?;
        Ok(into_gql(erg))
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct TradeSymbolInfo {
    pub symbol: models::TradeSymbol,
}

impl From<models::TradeSymbol> for TradeSymbolInfo {
    fn from(symbol: models::TradeSymbol) -> Self {
        TradeSymbolInfo { symbol }
    }
}

paginated_gql_object!(
    GQLTradeSymbolInfoPage,
    "TradeSymbolInfoPage",
    models::TradeSymbol,
    TradeSymbolInfo
);

#[async_graphql::ComplexObject]
impl TradeSymbolInfo {
    async fn requires(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLTradeSymbolInfoPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let regs = database::ExportImportMapping::get_imports_for_export(
            database_pool,
            self.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(regs.into())
    }

    async fn required_by(
        &self,
        ctx: &async_graphql::Context<'_>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<GQLTradeSymbolInfoPage> {
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let regs = database::ExportImportMapping::get_exports_for_import(
            database_pool,
            self.symbol,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(regs.into())
    }
}

pub struct ChartManagerInfo;
impl ChartManagerInfo {
    pub fn new() -> Self {
        ChartManagerInfo
    }
}

#[async_graphql::Object]
impl ChartManagerInfo {
    async fn busy(&self, ctx: &async_graphql::Context<'_>) -> Result<bool> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.chart_manager.is_busy())
    }

    async fn channel_state(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<crate::utils::ChannelInfo> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.chart_manager.get_channel_state())
    }
}

pub struct ConstructionManagerInfo;
impl ConstructionManagerInfo {
    pub fn new() -> Self {
        ConstructionManagerInfo
    }
}

#[async_graphql::Object]
impl ConstructionManagerInfo {
    async fn busy(&self, ctx: &async_graphql::Context<'_>) -> Result<bool> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.construction_manager.is_busy())
    }

    async fn channel_state(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<crate::utils::ChannelInfo> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.construction_manager.get_channel_state())
    }

    async fn running_shipments(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLConstructionShipment>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let shipments = context.construction_manager.get_running_shipments().await?;
        Ok(into_gql_vec(shipments))
    }
}

pub struct ContractManagerInfo;
impl ContractManagerInfo {
    pub fn new() -> Self {
        ContractManagerInfo
    }
}

#[async_graphql::Object]
impl ContractManagerInfo {
    async fn busy(&self, ctx: &async_graphql::Context<'_>) -> Result<bool> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.contract_manager.is_busy())
    }

    async fn channel_state(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<crate::utils::ChannelInfo> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.contract_manager.get_channel_state())
    }

    async fn running_shipments(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<GQLContractShipment>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let shipments = context.contract_manager.get_running_shipments().await?;
        Ok(into_gql_vec(shipments))
    }
}

pub struct FleetManagerInfo;
impl FleetManagerInfo {
    pub fn new() -> Self {
        FleetManagerInfo
    }
}

#[async_graphql::Object]
impl FleetManagerInfo {
    async fn busy(&self, ctx: &async_graphql::Context<'_>) -> Result<bool> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.fleet_manager.is_busy())
    }

    async fn channel_state(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<crate::utils::ChannelInfo> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.fleet_manager.get_channel_state())
    }
}

pub struct MiningManagerInfo;
impl MiningManagerInfo {
    pub fn new() -> Self {
        MiningManagerInfo
    }
}

#[async_graphql::Object]
impl MiningManagerInfo {
    async fn busy(&self, ctx: &async_graphql::Context<'_>) -> Result<bool> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.mining_manager.is_busy())
    }

    async fn channel_state(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<crate::utils::ChannelInfo> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.mining_manager.get_channel_state())
    }

    async fn get_assignments(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<Vec<MiningAssignment>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let assignments = context.mining_manager.get_assignments().await?;
        Ok(assignments
            .into_iter()
            .map(|f| MiningAssignment {
                waypoint_symbol: f.0,
                last_updated: f.1.get_last_updated(),
                assigned_ships: f
                    .1
                    .ship_iter()
                    .map(|f| AssignedShip {
                        ship_symbol: f.0.clone(),
                        level: *f.1,
                    })
                    .collect(),
            })
            .collect())
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
struct MiningAssignment {
    waypoint_symbol: String,
    assigned_ships: Vec<AssignedShip>,
    last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
struct AssignedShip {
    pub ship_symbol: String,
    pub level: crate::manager::mining_manager::AssignLevel,
}

#[async_graphql::ComplexObject]
impl MiningAssignment {
    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Option<GQLWaypoint>> {
        let data_loader = ctx.data::<DataLoader<database::WaypointLoader>>().unwrap();
        let erg = data_loader.load_one(self.waypoint_symbol.clone()).await?;
        Ok(into_gql(erg))
    }
}

#[async_graphql::ComplexObject]
impl AssignedShip {
    async fn ship(&self, ctx: &async_graphql::Context<'_>) -> Result<Option<GQLShip>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context.ship_manager.get_clone(&self.ship_symbol);
        Ok(ship.map(|s| s.into()))
    }
}

pub struct ScrappingManagerInfo;
impl ScrappingManagerInfo {
    pub fn new() -> Self {
        ScrappingManagerInfo
    }
}

#[async_graphql::Object]
impl ScrappingManagerInfo {
    async fn busy(&self, ctx: &async_graphql::Context<'_>) -> Result<bool> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.scrapping_manager.is_busy())
    }

    async fn channel_state(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<crate::utils::ChannelInfo> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.scrapping_manager.get_channel_state())
    }
    async fn possible_scraps(
        &self,
        ctx: &async_graphql::Context<'_>,
        ship_symbol: String,
    ) -> Result<Vec<ScrapInfo>> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        let ship = context.ship_manager.get_clone(&ship_symbol);

        if let Some(ship_clone) = ship {
            let waypoints = context.scrapping_manager.get_info(ship_clone).await?;
            Ok(waypoints
                .into_iter()
                .map(|f| ScrapInfo {
                    waypoint_symbol: f.0.clone(),
                    date: f.1,
                })
                .collect())
        } else {
            Ok(vec![])
        }
    }
}

#[derive(Debug, Clone, async_graphql::SimpleObject)]
struct ScrapInfo {
    waypoint_symbol: String,
    date: chrono::DateTime<chrono::Utc>,
}

pub struct TradeManagerInfo;
impl TradeManagerInfo {
    pub fn new() -> Self {
        TradeManagerInfo
    }
}

#[async_graphql::Object]
impl TradeManagerInfo {
    async fn busy(&self, ctx: &async_graphql::Context<'_>) -> Result<bool> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.trade_manager.is_busy())
    }

    async fn channel_state(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Result<crate::utils::ChannelInfo> {
        let context = ctx.data::<crate::utils::ConductorContext>().unwrap();
        Ok(context.trade_manager.get_channel_state())
    }
}
