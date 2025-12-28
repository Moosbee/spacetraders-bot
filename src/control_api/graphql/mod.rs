mod gql_models;
mod gql_ship;
pub mod mutations;

use async_graphql::Object;
use database::DatabaseConnector;
use space_traders_client::models;
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;
use utils::WaypointCan;

pub use gql_ship::AllShipLoader;

use crate::{
    control_api::graphql::gql_models::GQLShip,
    utils::{ConductorContext, RunInfo},
};

type Result<T> = std::result::Result<T, GraphiQLError>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn run_info<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<RunInfo> {
        let context = ctx.data::<ConductorContext>()?;
        let info = context.run_info.read().await.clone();
        Ok(info)
    }

    async fn api_counts<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<i64> {
        let context = ctx.data::<ConductorContext>()?;
        let counter = context.api.get_limiter().get_counter();
        Ok(counter)
    }

    async fn config<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<crate::utils::Config> {
        let context = ctx.data::<ConductorContext>()?;
        let config = context.config.read().await.clone();
        Ok(config)
    }

    async fn ship<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<GQLShip> {
        let context = ctx.data::<ConductorContext>()?;
        let ship = context
            .ship_manager
            .get_clone(&symbol)
            .ok_or(GraphiQLError::NotFound)?;
        Ok(ship.into())
    }

    async fn ships<'ctx>(&self, ctx: &async_graphql::Context<'ctx>) -> Result<Vec<GQLShip>> {
        let context = ctx.data::<ConductorContext>()?;
        let ships = context
            .ship_manager
            .get_all_clone()
            .await
            .into_values()
            .collect::<Vec<_>>();
        Ok(ships.into_iter().map(|s| s.into()).collect())
    }

    async fn market_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<MarketTransactionBy>,
    ) -> Result<Vec<gql_models::GQLMarketTransaction>> {
        let context = ctx.data::<ConductorContext>()?;
        let transactions = if let Some(by) = by {
            match by {
                MarketTransactionBy::Contract(contract_id) => {
                    database::MarketTransaction::get_by_contract(
                        &context.database_pool,
                        &contract_id,
                    )
                    .await
                }
                MarketTransactionBy::TradeRoute(trade_route_id) => {
                    database::MarketTransaction::get_by_trade_route(
                        &context.database_pool,
                        trade_route_id,
                    )
                    .await
                }
                MarketTransactionBy::Mining(mining_waypoint) => {
                    database::MarketTransaction::get_by_mining_waypoint(
                        &context.database_pool,
                        &mining_waypoint,
                    )
                    .await
                }
                MarketTransactionBy::Construction(construction_shipment) => {
                    database::MarketTransaction::get_by_construction(
                        &context.database_pool,
                        construction_shipment,
                    )
                    .await
                }
                MarketTransactionBy::Waypoint(waypoint_symbol) => {
                    database::MarketTransaction::get_by_waypoint(
                        &context.database_pool,
                        &waypoint_symbol,
                    )
                    .await
                }
                MarketTransactionBy::System(system_symbol) => {
                    database::MarketTransaction::get_by_system(
                        &context.database_pool,
                        &system_symbol,
                    )
                    .await
                }
                MarketTransactionBy::ShipSymbol(ship_symbol) => {
                    database::MarketTransaction::get_by_ship(&context.database_pool, &ship_symbol)
                        .await
                }
                MarketTransactionBy::TradeSymbol(trade_symbol) => {
                    database::MarketTransaction::get_by_trade_symbol(
                        &context.database_pool,
                        trade_symbol,
                    )
                    .await
                }
                MarketTransactionBy::Type(transaction_type) => {
                    database::MarketTransaction::get_by_trade_type(
                        &context.database_pool,
                        transaction_type,
                    )
                    .await
                }
            }
        } else {
            database::MarketTransaction::get_all(&context.database_pool).await
        }?;
        Ok(transactions.into_iter().map(Into::into).collect())
    }

    async fn shipyard_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ShipyardTransactionBy>,
    ) -> Result<Vec<gql_models::GQLShipyardTransaction>> {
        let context = ctx.data::<ConductorContext>()?;
        let transactions = if let Some(by) = by {
            match by {
                ShipyardTransactionBy::Waypoint(waypoint_symbol) => {
                    database::ShipyardTransaction::get_by_waypoint(
                        &context.database_pool,
                        &waypoint_symbol,
                    )
                    .await
                }
                ShipyardTransactionBy::System(system_symbol) => {
                    database::ShipyardTransaction::get_by_system(
                        &context.database_pool,
                        &system_symbol,
                    )
                    .await
                }
                ShipyardTransactionBy::Type(ship_type) => {
                    database::ShipyardTransaction::get_by_ship_type(
                        &context.database_pool,
                        ship_type,
                    )
                    .await
                }
                ShipyardTransactionBy::Agent(agent_symbol) => {
                    database::ShipyardTransaction::get_by_agent(
                        &context.database_pool,
                        &agent_symbol,
                    )
                    .await
                }
            }
        } else {
            database::ShipyardTransaction::get_all(&context.database_pool).await
        }?;
        Ok(transactions.into_iter().map(Into::into).collect())
    }

    async fn chart_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        ship_symbol: Option<String>,
    ) -> Result<Vec<gql_models::GQLChartTransaction>> {
        let context = ctx.data::<ConductorContext>()?;
        let transactions = if let Some(ship_symbol) = ship_symbol {
            database::ChartTransaction::get_by_ship_symbol(&context.database_pool, &ship_symbol)
                .await?
        } else {
            database::ChartTransaction::get_all(&context.database_pool).await?
        };
        Ok(transactions.into_iter().map(Into::into).collect())
    }

    async fn repair_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLRepairTransaction>> {
        let context = ctx.data::<ConductorContext>()?;
        let transactions = database::RepairTransaction::get_all(&context.database_pool).await?;
        Ok(transactions.into_iter().map(Into::into).collect())
    }

    async fn scrap_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLScrapTransaction>> {
        let context = ctx.data::<ConductorContext>()?;
        let transactions = database::ScrapTransaction::get_all(&context.database_pool).await?;
        Ok(transactions.into_iter().map(Into::into).collect())
    }

    async fn ship_modification_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLShipModificationTransaction>> {
        let context = ctx.data::<ConductorContext>()?;
        let transactions =
            database::ShipModificationTransaction::get_all(&context.database_pool).await?;
        Ok(transactions.into_iter().map(Into::into).collect())
    }

    async fn waypoint<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<gql_models::GQLWaypoint> {
        let data_loader =
            ctx.data::<async_graphql::dataloader::DataLoader<database::WaypointLoader>>()?;
        let waypoint = data_loader
            .load_one(symbol.clone())
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(waypoint.into())
    }

    async fn waypoints<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLWaypoint>> {
        let context = ctx.data::<ConductorContext>()?;
        let waypoints = database::Waypoint::get_all(&context.database_pool).await?;
        Ok(waypoints.into_iter().map(Into::into).collect())
    }

    async fn system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<gql_models::GQLSystem> {
        let context = ctx.data::<ConductorContext>()?;
        let system = database::System::get_by_symbol(&context.database_pool, &symbol)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(system.into())
    }

    async fn systems<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        only_with_fleets_or_ships: Option<bool>,
    ) -> Result<Vec<gql_models::GQLSystem>> {
        let context = ctx.data::<ConductorContext>()?;
        let all_systems = database::System::get_all(&context.database_pool).await?;

        let systems = if only_with_fleets_or_ships.unwrap_or(false) {
            let fleets = database::Fleet::get_all(&context.database_pool).await?;
            let ships = context.ship_manager.get_all_clone().await;
            let mut filter_systems = fleets
                .iter()
                .map(|f| f.system_symbol.clone())
                .collect::<HashSet<_>>();
            filter_systems.extend(ships.values().map(|f| f.nav.system_symbol.clone()));

            all_systems
                .into_iter()
                .filter(|s| filter_systems.contains(&s.symbol))
                .collect()
        } else {
            all_systems
        };

        Ok(systems.into_iter().map(Into::into).collect())
    }

    async fn agent<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<gql_models::GQLAgent> {
        let context = ctx.data::<ConductorContext>()?;
        let agent = database::Agent::get_last_by_symbol(&context.database_pool, &symbol)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(agent.into())
    }

    async fn agent_history<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<Vec<gql_models::GQLAgent>> {
        let context = ctx.data::<ConductorContext>()?;
        let agent = database::Agent::get_by_symbol(&context.database_pool, &symbol).await?;
        Ok(agent.into_iter().map(Into::into).collect())
    }

    async fn agents<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLAgent>> {
        let context = ctx.data::<ConductorContext>()?;
        let agents = database::Agent::get_last(&context.database_pool).await?;
        Ok(agents.into_iter().map(Into::into).collect())
    }

    async fn contract<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<gql_models::GQLContract> {
        let context = ctx.data::<ConductorContext>()?;
        let contract = database::Contract::get_by_id(&context.database_pool, &symbol)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(contract.into())
    }

    async fn contracts<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ContractBy>,
    ) -> Result<Vec<gql_models::GQLContract>> {
        let context = ctx.data::<ConductorContext>()?;
        let contracts = if let Some(by) = by {
            match by {
                ContractBy::Faction(symbol) => {
                    database::Contract::get_by_faction_symbol(
                        &context.database_pool,
                        &symbol.to_string(),
                    )
                    .await?
                }
            }
        } else {
            database::Contract::get_all(&context.database_pool).await?
        };
        Ok(contracts.into_iter().map(Into::into).collect())
    }

    async fn contract_deliveries<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ContractDeliveryBy>,
    ) -> Result<Vec<gql_models::GQLContractDelivery>> {
        let context = ctx.data::<ConductorContext>()?;
        let contract_deliveries = if let Some(by) = by {
            match by {
                ContractDeliveryBy::Contract(id) => {
                    database::ContractDelivery::get_by_contract_id(&context.database_pool, &id)
                        .await?
                }
                ContractDeliveryBy::TradeSymbol(symbol) => {
                    database::ContractDelivery::get_by_trade_symbol(&context.database_pool, &symbol)
                        .await?
                }
                ContractDeliveryBy::Waypoint(symbol) => {
                    database::ContractDelivery::get_by_destination_symbol(
                        &context.database_pool,
                        &symbol,
                    )
                    .await?
                }
            }
        } else {
            database::ContractDelivery::get_all(&context.database_pool).await?
        };
        Ok(contract_deliveries.into_iter().map(Into::into).collect())
    }

    async fn contract_shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ContractShipmentBy>,
    ) -> Result<Vec<gql_models::GQLContractShipment>> {
        let context = ctx.data::<ConductorContext>()?;
        let contract_shipments = if let Some(by) = by {
            match by {
                ContractShipmentBy::Contract(id) => {
                    database::ContractShipment::get_by_contract_id(&context.database_pool, &id)
                        .await
                }
                ContractShipmentBy::TradeSymbol(symbol) => {
                    database::ContractShipment::get_by_trade_symbol(&context.database_pool, &symbol)
                        .await
                }
                ContractShipmentBy::SourceWaypoint(source_symbol) => {
                    database::ContractShipment::get_by_source_symbol(
                        &context.database_pool,
                        &source_symbol,
                    )
                    .await
                }
                ContractShipmentBy::DestinationWaypoint(destination_symbol) => {
                    database::ContractShipment::get_by_destination_symbol(
                        &context.database_pool,
                        &destination_symbol,
                    )
                    .await
                }
                ContractShipmentBy::ShipSymbol(symbol) => {
                    database::ContractShipment::get_by_ship(&context.database_pool, &symbol).await
                }
            }
        } else {
            database::ContractShipment::get_all(&context.database_pool).await
        }?;
        Ok(contract_shipments.into_iter().map(Into::into).collect())
    }

    async fn extraction<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: i64,
    ) -> Result<gql_models::GQLExtraction> {
        let context = ctx.data::<ConductorContext>()?;
        let extraction = database::Extraction::get_by_id(&context.database_pool, symbol)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(extraction.into())
    }

    async fn extractions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ExtractionBy>,
    ) -> Result<Vec<gql_models::GQLExtraction>> {
        let context = ctx.data::<ConductorContext>()?;
        let extractions = if let Some(by) = by {
            match by {
                ExtractionBy::Waypoint(symbol) => {
                    database::Extraction::get_by_waypoint_symbol(
                        &context.database_pool,
                        &symbol.to_string(),
                    )
                    .await
                }
                ExtractionBy::System(symbol) => {
                    database::Extraction::get_by_system_symbol(
                        &context.database_pool,
                        &symbol.to_string(),
                    )
                    .await
                }
                ExtractionBy::ShipSymbol(symbol) => {
                    database::Extraction::get_by_ship(&context.database_pool, &symbol.to_string())
                        .await
                }
                ExtractionBy::TradeSymbol(symbol) => {
                    database::Extraction::get_by_trade_symbol(&context.database_pool, &symbol).await
                }
                ExtractionBy::Siphon(siphon) => {
                    database::Extraction::get_by_siphon(&context.database_pool, siphon).await
                }
                ExtractionBy::Survey(symbol) => {
                    database::Extraction::get_by_survey_symbol(
                        &context.database_pool,
                        &symbol.to_string(),
                    )
                    .await
                }
            }
        } else {
            database::Extraction::get_all(&context.database_pool).await
        }?;
        Ok(extractions.into_iter().map(Into::into).collect())
    }

    async fn fleets<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<FleetBy>,
    ) -> Result<Vec<gql_models::GQLFleet>> {
        let context = ctx.data::<ConductorContext>()?;
        let fleets = if let Some(by) = by {
            match by {
                FleetBy::System(system_symbol) => {
                    database::Fleet::get_by_system(&context.database_pool, &system_symbol).await
                }
                FleetBy::Type(fleet_type) => {
                    database::Fleet::get_by_type(&context.database_pool, fleet_type).await
                }
            }
        } else {
            database::Fleet::get_all(&context.database_pool).await
        }?;
        Ok(fleets.into_iter().map(Into::into).collect())
    }

    async fn ship_assignments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ShipAssignmentBy>,
    ) -> Result<Vec<gql_models::GQLShipAssignment>> {
        let context = ctx.data::<ConductorContext>()?;
        let ship_assignments = if let Some(by) = by {
            match by {
                ShipAssignmentBy::Fleet(fleet_id) => {
                    database::ShipAssignment::get_by_fleet_id(&context.database_pool, fleet_id)
                        .await
                }
                ShipAssignmentBy::Open(_assigned) => {
                    database::ShipAssignment::get_open_assignments(&context.database_pool).await
                }
            }
        } else {
            database::ShipAssignment::get_all(&context.database_pool).await
        }?;
        Ok(ship_assignments.into_iter().map(Into::into).collect())
    }

    async fn reserved_funds<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLReservedFund>> {
        let context = ctx.data::<ConductorContext>()?;
        let reserved_funds = database::ReservedFund::get_all(&context.database_pool).await?;
        Ok(reserved_funds.into_iter().map(Into::into).collect())
    }

    async fn surveys<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<SurveyBy>,
    ) -> Result<Vec<gql_models::GQLSurvey>> {
        let context = ctx.data::<ConductorContext>()?;
        let surveys = if let Some(by) = by {
            match by {
                SurveyBy::Waypoint(waypoint_symbol) => {
                    database::Survey::get_by_waypoint_symbol(
                        &context.database_pool,
                        &waypoint_symbol,
                    )
                    .await
                }
                SurveyBy::System(system_symbol) => {
                    database::Survey::get_by_system_symbol(&context.database_pool, &system_symbol)
                        .await
                }
                SurveyBy::Size(size) => {
                    database::Survey::get_by_size(&context.database_pool, size).await
                }
                SurveyBy::ShipSymbol(ship_symbol) => {
                    database::Survey::get_by_ship(&context.database_pool, &ship_symbol).await
                }
            }
        } else {
            database::Survey::get_all(&context.database_pool).await
        }?;
        Ok(surveys.into_iter().map(Into::into).collect())
    }

    async fn survey<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        signature: String,
    ) -> Result<gql_models::GQLSurvey> {
        let context = ctx.data::<ConductorContext>()?;
        let survey = database::Survey::get_by_signature(&context.database_pool, &signature).await?;
        Ok(survey.into())
    }

    async fn trade_routes<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLTradeRoute>> {
        let context = ctx.data::<ConductorContext>()?;
        let trade_routes = database::TradeRoute::get_all(&context.database_pool).await?;
        Ok(trade_routes.into_iter().map(Into::into).collect())
    }

    async fn trade_route<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        route_id: i32,
    ) -> Result<gql_models::GQLTradeRoute> {
        let context = ctx.data::<ConductorContext>()?;
        let trade_route = database::TradeRoute::get_by_id(&context.database_pool, route_id)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(trade_route.into())
    }

    async fn ship_infos<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLShipInfo>> {
        let context = ctx.data::<ConductorContext>()?;
        let ship_info = database::ShipInfo::get_all(&context.database_pool).await?;
        Ok(ship_info.into_iter().map(Into::into).collect())
    }

    async fn ship_info<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<gql_models::GQLShipInfo> {
        let context = ctx.data::<ConductorContext>()?;
        let ship_info = database::ShipInfo::get_by_symbol(&context.database_pool, &symbol)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(ship_info.into())
    }

    async fn ship_states<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ShipStateBy>,
    ) -> Result<Vec<gql_models::GQLShipState>> {
        let context = ctx.data::<ConductorContext>()?;
        let ship_states = if let Some(by) = by {
            match by {
                ShipStateBy::Waypoint(waypoint) => {
                    database::ShipState::get_by_waypoint(&context.database_pool, &waypoint).await
                }
                ShipStateBy::System(system) => {
                    database::ShipState::get_by_system(&context.database_pool, &system).await
                }
                ShipStateBy::ShipSymbol(symbol) => {
                    database::ShipState::get_by_ship(&context.database_pool, &symbol).await
                }
            }
        } else {
            database::ShipState::get_all(&context.database_pool).await
        }?;
        Ok(ship_states.into_iter().map(Into::into).collect())
    }

    async fn shipyards<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLShipyard>> {
        let context = ctx.data::<ConductorContext>()?;
        let shipyards = database::Shipyard::get_last(&context.database_pool).await?;
        Ok(shipyards.into_iter().map(Into::into).collect())
    }

    async fn shipyard<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<gql_models::GQLShipyard> {
        let context = ctx.data::<ConductorContext>()?;
        let shipyard = database::Shipyard::get_last_by_waypoint(&context.database_pool, &symbol)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(shipyard.into())
    }

    async fn shipyard_ships<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ShipyardShipBy>,
    ) -> Result<Vec<gql_models::GQLShipyardShip>> {
        let context = ctx.data::<ConductorContext>()?;
        let shipyard_ships = if let Some(by) = by {
            match by {
                ShipyardShipBy::Waypoint(waypoint) => {
                    database::ShipyardShip::get_last_by_waypoint(&context.database_pool, &waypoint)
                        .await
                }
                ShipyardShipBy::System(system) => {
                    database::ShipyardShip::get_last_by_system(&context.database_pool, &system)
                        .await
                }
                ShipyardShipBy::ShipSymbol(symbol) => {
                    database::ShipyardShip::get_last_by_ship_symbol(&context.database_pool, &symbol)
                        .await
                }
            }
        } else {
            database::ShipyardShip::get_last(&context.database_pool).await
        }?;
        Ok(shipyard_ships.into_iter().map(Into::into).collect())
    }

    async fn construction_materials<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ConstructionMaterialBy>,
    ) -> Result<Vec<gql_models::GQLConstructionMaterial>> {
        let context = ctx.data::<ConductorContext>()?;
        let construction_materials = if let Some(by) = by {
            match by {
                ConstructionMaterialBy::Waypoint(waypoint) => {
                    database::ConstructionMaterial::get_by_waypoint(
                        &context.database_pool,
                        &waypoint,
                    )
                    .await
                }
                ConstructionMaterialBy::System(system) => {
                    database::ConstructionMaterial::get_by_system(&context.database_pool, &system)
                        .await
                }
                ConstructionMaterialBy::TradeSymbol(trade_symbol) => {
                    database::ConstructionMaterial::get_by_trade_symbol(
                        &context.database_pool,
                        &trade_symbol,
                    )
                    .await
                }
            }
        } else {
            database::ConstructionMaterial::get_all(&context.database_pool).await
        }?;
        Ok(construction_materials.into_iter().map(Into::into).collect())
    }

    async fn construction_shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ConstructionShipmentBy>,
    ) -> Result<Vec<gql_models::GQLConstructionShipment>> {
        let context = ctx.data::<ConductorContext>()?;
        let construction_shipments = if let Some(by) = by {
            match by {
                ConstructionShipmentBy::Waypoint(waypoint) => {
                    database::ConstructionShipment::get_by_destination_waypoint(
                        &context.database_pool,
                        &waypoint,
                    )
                    .await
                }
                ConstructionShipmentBy::System(system) => {
                    database::ConstructionShipment::get_by_system(&context.database_pool, &system)
                        .await
                }
                ConstructionShipmentBy::TradeSymbol(trade_symbol) => {
                    database::ConstructionShipment::get_by_trade_symbol(
                        &context.database_pool,
                        &trade_symbol,
                    )
                    .await
                }
                ConstructionShipmentBy::Material(material_id) => {
                    database::ConstructionShipment::get_by_material_id(
                        &context.database_pool,
                        material_id,
                    )
                    .await
                }
                ConstructionShipmentBy::ShipSymbol(ship_symbol) => {
                    database::ConstructionShipment::get_by_ship(
                        &context.database_pool,
                        &ship_symbol,
                    )
                    .await
                }
            }
        } else {
            database::ConstructionShipment::get_all(&context.database_pool).await
        }?;
        Ok(construction_shipments.into_iter().map(Into::into).collect())
    }

    async fn jump_gate_connections<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        from: Option<String>,
    ) -> Result<Vec<gql_models::GQLJumpGateConnection>> {
        let context = ctx.data::<ConductorContext>()?;
        let jump_gate_connections = if let Some(from) = from {
            database::JumpGateConnection::get_all_from(&context.database_pool, &from).await?
        } else {
            database::JumpGateConnection::get_all(&context.database_pool).await?
        };
        Ok(jump_gate_connections.into_iter().map(Into::into).collect())
    }

    async fn jump_connections<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GateConn>> {
        let context = ctx.data::<ConductorContext>()?;
        let connections = database::JumpGateConnection::get_all(&context.database_pool).await?;

        let mut connection_map: HashMap<(String, String), gql_models::GateConn> = HashMap::new();

        for connection in connections {
            let mut pair = [connection.from.clone(), connection.to.clone()];
            pair.sort(); // Ensure the pair is always in a consistent order
            let entry = connection_map.entry((pair[0].clone(), pair[1].clone()));

            let entry = entry.or_insert_with(|| gql_models::GateConn {
                point_a_symbol: pair[0].clone(),
                point_b_symbol: pair[1].clone(),
                under_construction_a: false,
                under_construction_b: false,
                from_a: false,
                from_b: false,
            });
            let is_from_a = connection.from == pair[0];
            let is_from_b = connection.from == pair[1];
            if is_from_a {
                entry.from_a = true;
            } else if is_from_b {
                entry.from_b = true;
            }
        }

        let gate_waypoints = database::Waypoint::get_all(&context.database_pool)
            .await?
            .into_iter()
            .filter(|w| w.is_jump_gate())
            .map(|w| (w.symbol.clone(), w))
            .collect::<HashMap<_, _>>();

        for connection in connection_map.values_mut() {
            connection.under_construction_a = gate_waypoints
                .get(&connection.point_a_symbol)
                .map(|w| w.is_under_construction)
                .unwrap_or(false);
            connection.under_construction_b = gate_waypoints
                .get(&connection.point_b_symbol)
                .map(|w| w.is_under_construction)
                .unwrap_or(false);
        }

        Ok(connection_map.into_values().collect())
    }

    async fn market_trades<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<MarketTradeBy>,
    ) -> Result<Vec<gql_models::GQLMarketTrade>> {
        let context = ctx.data::<ConductorContext>()?;
        let market_trades = if let Some(by) = by {
            match by {
                MarketTradeBy::Waypoint(waypoint) => {
                    database::MarketTrade::get_last_by_waypoint(&context.database_pool, &waypoint)
                        .await
                }
                MarketTradeBy::TradeSymbol(trade_symbol) => {
                    database::MarketTrade::get_last_by_symbol(&context.database_pool, &trade_symbol)
                        .await
                }
                MarketTradeBy::System(system) => {
                    database::MarketTrade::get_last_by_system(&context.database_pool, &system).await
                }
            }
        } else {
            database::MarketTrade::get_last(&context.database_pool).await
        }?;
        Ok(market_trades.into_iter().map(Into::into).collect())
    }

    async fn market_trade_goods<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<MarketTradeGoodBy>,
    ) -> Result<Vec<gql_models::GQLMarketTradeGood>> {
        let context = ctx.data::<ConductorContext>()?;
        let market_trade_goods = if let Some(by) = by {
            match by {
                MarketTradeGoodBy::Waypoint(waypoint) => {
                    database::MarketTradeGood::get_last_by_waypoint(
                        &context.database_pool,
                        &waypoint,
                    )
                    .await
                }
                MarketTradeGoodBy::TradeSymbol(trade_symbol) => {
                    database::MarketTradeGood::get_last_by_symbol(
                        &context.database_pool,
                        &trade_symbol,
                    )
                    .await
                }
                MarketTradeGoodBy::System(system) => {
                    database::MarketTradeGood::get_last_by_system(&context.database_pool, &system)
                        .await
                }
            }
        } else {
            database::MarketTradeGood::get_all(&context.database_pool).await
        }?;
        Ok(market_trade_goods.into_iter().map(Into::into).collect())
    }

    async fn trade_symbol_infos(&self) -> Result<Vec<gql_models::TradeSymbolInfo>> {
        let trade_symbol_infos = models::TradeSymbol::iter().map(Into::into).collect();
        Ok(trade_symbol_infos)
    }

    async fn ship_routes<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLRoute>> {
        // Changed return type to GQLRoute
        let database_pool = ctx.data::<database::DbPool>().unwrap();
        let reg = database::Route::get_all(database_pool).await?;
        Ok(reg.into_iter().map(Into::into).collect()) // Added conversion
    }

    async fn budget<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<crate::manager::budget_manager::BudgetInfo> {
        let context = ctx.data::<ConductorContext>()?;
        let budget_info: crate::manager::budget_manager::BudgetInfo =
            context.budget_manager.get_budget_info().await;
        Ok(budget_info)
    }

    async fn chart_manager(&self) -> gql_models::ChartManagerInfo {
        gql_models::ChartManagerInfo::new()
    }
    async fn construction_manager(&self) -> gql_models::ConstructionManagerInfo {
        gql_models::ConstructionManagerInfo::new()
    }
    async fn contract_manager(&self) -> gql_models::ContractManagerInfo {
        gql_models::ContractManagerInfo::new()
    }
    async fn fleet_manager(&self) -> gql_models::FleetManagerInfo {
        gql_models::FleetManagerInfo::new()
    }
    async fn mining_manager(&self) -> gql_models::MiningManagerInfo {
        gql_models::MiningManagerInfo::new()
    }
    async fn scrapping_manager(&self) -> gql_models::ScrappingManagerInfo {
        gql_models::ScrappingManagerInfo::new()
    }
    async fn trade_manager(&self) -> gql_models::TradeManagerInfo {
        gql_models::TradeManagerInfo::new()
    }
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum MarketTradeGoodBy {
    Waypoint(String),
    TradeSymbol(models::TradeSymbol),
    System(String),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum MarketTradeBy {
    Waypoint(String),
    TradeSymbol(models::TradeSymbol),
    System(String),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ContractDeliveryBy {
    Waypoint(String),
    TradeSymbol(models::TradeSymbol),
    Contract(String),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ContractShipmentBy {
    SourceWaypoint(String),
    DestinationWaypoint(String),
    TradeSymbol(models::TradeSymbol),
    Contract(String),
    ShipSymbol(String),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ConstructionMaterialBy {
    Waypoint(String),
    System(String),
    TradeSymbol(models::TradeSymbol),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ConstructionShipmentBy {
    Waypoint(String),
    System(String),
    TradeSymbol(models::TradeSymbol),
    Material(i64),
    ShipSymbol(String),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ShipyardShipBy {
    Waypoint(String),
    System(String),
    ShipSymbol(models::ShipType),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ShipStateBy {
    Waypoint(String),
    System(String),
    ShipSymbol(String),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum SurveyBy {
    Waypoint(String),
    System(String),
    Size(models::SurveySize),
    ShipSymbol(String),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum FleetBy {
    System(String),
    Type(database::FleetType),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ShipAssignmentBy {
    Fleet(i32),
    Open(bool),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ExtractionBy {
    Waypoint(String),
    System(String),
    ShipSymbol(String),
    TradeSymbol(models::TradeSymbol),
    Siphon(bool),
    Survey(String),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ContractBy {
    Faction(models::FactionSymbol),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum ShipyardTransactionBy {
    Waypoint(String),
    System(String),
    Type(models::ShipType),
    Agent(String),
}

#[derive(Debug, Clone, async_graphql::OneofObject)]
enum MarketTransactionBy {
    Contract(String),
    TradeRoute(i32),
    Mining(String),
    Construction(i64),
    Waypoint(String),
    System(String),
    ShipSymbol(String),
    TradeSymbol(models::TradeSymbol),
    Type(models::market_transaction::Type),
}

#[derive(thiserror::Error, Debug)]
pub enum GraphiQLError {
    #[error("Not found")]
    NotFound,
    #[error("Graphql error: {:?}", 0)]
    GraphiQL(async_graphql::Error),
    #[error("Database error: {0}")]
    Database(#[from] database::Error),
    #[error("ArcDatabase error: {0}")]
    ArcDatabase(#[from] std::sync::Arc<database::Error>),
    #[error("IO error: {0}")]
    IO(String),
}

impl From<async_graphql::Error> for GraphiQLError {
    fn from(value: async_graphql::Error) -> Self {
        GraphiQLError::GraphiQL(value)
    }
}

// todo implement DataLoaders for, individual Waypoints, system waypoints, individual fleets, system fleets
