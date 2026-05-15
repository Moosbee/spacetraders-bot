mod gql_models;
mod gql_ship;
pub mod mutations;

use async_graphql::{Object, Subscription};
use database::DatabaseConnectorAsync;
use futures::{Stream, StreamExt};
use space_traders_client::models;
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;
use tokio_stream::wrappers::BroadcastStream;
use utils::WaypointCan;

pub use gql_ship::AllShipLoader;

use crate::{
    control_api::graphql::gql_models::GQLShip,
    utils::{ConductorContext, RunInfo},
};

type Result<T> = std::result::Result<T, GraphiQLError>;

fn paginated_query(page: Option<i64>, page_size: Option<i64>) -> database::PaginatedQuery {
    database::PaginatedQuery::new(page.unwrap_or(1), page_size)
}

pub struct QueryRoot;

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn ship_events<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        ship_symbol: Option<String>,
    ) -> Result<impl Stream<Item = gql_models::GQLShipEvent>> {
        let context = ctx.data::<ConductorContext>()?;
        let receiver = context.ship_manager.get_event_rx();

        Ok(BroadcastStream::new(receiver).filter_map(move |result| {
            let ship_symbol = ship_symbol.clone();
            async move {
                match result {
                    Ok(event) => {
                        let include = ship_symbol
                            .as_ref()
                            .map(|symbol| symbol == &event.ship_symbol)
                            .unwrap_or(true);
                        if include {
                            Some(event.into())
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            }
        }))
    }
}

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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLMarketTransactionPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let transactions = if let Some(by) = by {
            match by {
                MarketTransactionBy::Contract(contract_id) => {
                    database::MarketTransaction::get_by_contract(
                        &context.database_pool,
                        &contract_id,
                        query,
                    )
                    .await
                }
                MarketTransactionBy::TradeRoute(trade_route_id) => {
                    database::MarketTransaction::get_by_trade_route(
                        &context.database_pool,
                        trade_route_id,
                        query,
                    )
                    .await
                }
                MarketTransactionBy::Mining(mining_waypoint) => {
                    database::MarketTransaction::get_by_mining_waypoint(
                        &context.database_pool,
                        &mining_waypoint,
                        query,
                    )
                    .await
                }
                MarketTransactionBy::Construction(construction_shipment) => {
                    database::MarketTransaction::get_by_construction(
                        &context.database_pool,
                        construction_shipment,
                        query,
                    )
                    .await
                }
                MarketTransactionBy::Waypoint(waypoint_symbol) => {
                    database::MarketTransaction::get_by_waypoint(
                        &context.database_pool,
                        &waypoint_symbol,
                        query,
                    )
                    .await
                }
                MarketTransactionBy::System(system_symbol) => {
                    database::MarketTransaction::get_by_system(
                        &context.database_pool,
                        &system_symbol,
                        query,
                    )
                    .await
                }
                MarketTransactionBy::ShipSymbol(ship_symbol) => {
                    database::MarketTransaction::get_by_ship(
                        &context.database_pool,
                        &ship_symbol,
                        query,
                    )
                    .await
                }
                MarketTransactionBy::TradeSymbol(trade_symbol) => {
                    database::MarketTransaction::get_by_trade_symbol(
                        &context.database_pool,
                        trade_symbol,
                        query,
                    )
                    .await
                }
                MarketTransactionBy::Type(transaction_type) => {
                    database::MarketTransaction::get_by_trade_type(
                        &context.database_pool,
                        transaction_type,
                        query,
                    )
                    .await
                }
            }
        } else {
            database::MarketTransaction::get_all(&context.database_pool, query).await
        }?;
        Ok(transactions.into())
    }

    async fn shipyard_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ShipyardTransactionBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLShipyardTransactionPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let transactions = if let Some(by) = by {
            match by {
                ShipyardTransactionBy::Waypoint(waypoint_symbol) => {
                    database::ShipyardTransaction::get_by_waypoint(
                        &context.database_pool,
                        &waypoint_symbol,
                        query,
                    )
                    .await
                }
                ShipyardTransactionBy::System(system_symbol) => {
                    database::ShipyardTransaction::get_by_system(
                        &context.database_pool,
                        &system_symbol,
                        query,
                    )
                    .await
                }
                ShipyardTransactionBy::Type(ship_type) => {
                    database::ShipyardTransaction::get_by_ship_type(
                        &context.database_pool,
                        ship_type,
                        query,
                    )
                    .await
                }
                ShipyardTransactionBy::Agent(agent_symbol) => {
                    database::ShipyardTransaction::get_by_agent(
                        &context.database_pool,
                        &agent_symbol,
                        query,
                    )
                    .await
                }
            }
        } else {
            database::ShipyardTransaction::get_all(&context.database_pool, query).await
        }?;
        Ok(transactions.into())
    }

    async fn chart_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        ship_symbol: Option<String>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLChartTransactionPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let transactions = if let Some(ship_symbol) = ship_symbol {
            database::ChartTransaction::get_by_ship_symbol(
                &context.database_pool,
                &ship_symbol,
                query,
            )
            .await?
        } else {
            database::ChartTransaction::get_all(&context.database_pool, query).await?
        };
        Ok(transactions.into())
    }

    async fn repair_transactions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLRepairTransactionPage> {
        let context = ctx.data::<ConductorContext>()?;
        let transactions = database::RepairTransaction::get_all(
            &context.database_pool,
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
    ) -> Result<gql_models::GQLScrapTransactionPage> {
        let context = ctx.data::<ConductorContext>()?;
        let transactions = database::ScrapTransaction::get_all(
            &context.database_pool,
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
    ) -> Result<gql_models::GQLShipModificationTransactionPage> {
        let context = ctx.data::<ConductorContext>()?;
        let transactions = database::ShipModificationTransaction::get_all(
            &context.database_pool,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(transactions.into())
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLWaypointPage> {
        let context = ctx.data::<ConductorContext>()?;
        let waypoints =
            database::Waypoint::get_all(&context.database_pool, paginated_query(page, page_size))
                .await?;
        Ok(waypoints.into())
    }

    async fn system<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<gql_models::GQLSystem> {
        let context = ctx.data::<ConductorContext>()?;
        let system = database::System::get_by_id(&context.database_pool, &symbol)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(system.into())
    }

    async fn systems<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        only_with_fleets_or_ships: Option<bool>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLSystemPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);

        if only_with_fleets_or_ships.unwrap_or(false) {
            let all_systems = database::System::get_all(
                &context.database_pool,
                database::PaginatedQuery::unpaged(),
            )
            .await?
            .items;
            let fleets = database::Fleet::get_all(
                &context.database_pool,
                database::PaginatedQuery::unpaged(),
            )
            .await?
            .items;
            let ships = context.ship_manager.get_all_clone().await;
            let mut filter_systems = fleets
                .iter()
                .map(|f| f.system_symbol.clone())
                .collect::<HashSet<_>>();
            filter_systems.extend(ships.values().map(|f| f.nav.system_symbol.clone()));

            let systems = all_systems
                .into_iter()
                .filter(|s| filter_systems.contains(&s.symbol))
                .collect();
            Ok(database::paginate_items(query, systems)?.into())
        } else {
            Ok(database::System::get_all(&context.database_pool, query)
                .await?
                .into())
        }
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
        let agent = database::Agent::get_by_symbol(
            &context.database_pool,
            &symbol,
            database::PaginatedQuery::unpaged(),
        )
        .await?;
        Ok(agent.items.into_iter().map(Into::into).collect())
    }

    async fn agents<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
    ) -> Result<Vec<gql_models::GQLAgent>> {
        let context = ctx.data::<ConductorContext>()?;
        let agents =
            database::Agent::get_last(&context.database_pool, database::PaginatedQuery::unpaged())
                .await?;
        Ok(agents.items.into_iter().map(Into::into).collect())
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLContractPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let contracts = if let Some(by) = by {
            match by {
                ContractBy::Faction(symbol) => {
                    database::Contract::get_by_faction_symbol(
                        &context.database_pool,
                        &symbol.to_string(),
                        query,
                    )
                    .await?
                }
            }
        } else {
            database::Contract::get_all(&context.database_pool, query).await?
        };
        Ok(contracts.into())
    }

    async fn contract_deliveries<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ContractDeliveryBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLContractDeliveryPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let contract_deliveries = if let Some(by) = by {
            match by {
                ContractDeliveryBy::Contract(id) => {
                    database::ContractDelivery::get_by_contract_id(
                        &context.database_pool,
                        &id,
                        query,
                    )
                    .await?
                }
                ContractDeliveryBy::TradeSymbol(symbol) => {
                    database::ContractDelivery::get_by_trade_symbol(
                        &context.database_pool,
                        &symbol,
                        query,
                    )
                    .await?
                }
                ContractDeliveryBy::Waypoint(symbol) => {
                    database::ContractDelivery::get_by_destination_symbol(
                        &context.database_pool,
                        &symbol,
                        query,
                    )
                    .await?
                }
            }
        } else {
            database::ContractDelivery::get_all(&context.database_pool, query).await?
        };
        Ok(contract_deliveries.into())
    }

    async fn contract_shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ContractShipmentBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLContractShipmentPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let contract_shipments = if let Some(by) = by {
            match by {
                ContractShipmentBy::Contract(id) => {
                    database::ContractShipment::get_by_contract_id(
                        &context.database_pool,
                        &id,
                        query,
                    )
                    .await
                }
                ContractShipmentBy::TradeSymbol(symbol) => {
                    database::ContractShipment::get_by_trade_symbol(
                        &context.database_pool,
                        &symbol,
                        query,
                    )
                    .await
                }
                ContractShipmentBy::SourceWaypoint(source_symbol) => {
                    database::ContractShipment::get_by_source_symbol(
                        &context.database_pool,
                        &source_symbol,
                        query,
                    )
                    .await
                }
                ContractShipmentBy::DestinationWaypoint(destination_symbol) => {
                    database::ContractShipment::get_by_destination_symbol(
                        &context.database_pool,
                        &destination_symbol,
                        query,
                    )
                    .await
                }
                ContractShipmentBy::ShipSymbol(symbol) => {
                    database::ContractShipment::get_by_ship(&context.database_pool, &symbol, query)
                        .await
                }
            }
        } else {
            database::ContractShipment::get_all(&context.database_pool, query).await
        }?;
        Ok(contract_shipments.into())
    }

    async fn extraction<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: i64,
    ) -> Result<gql_models::GQLExtraction> {
        let context = ctx.data::<ConductorContext>()?;
        let extraction = database::Extraction::get_by_id(&context.database_pool, &symbol)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(extraction.into())
    }

    async fn extractions<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ExtractionBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLExtractionPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let extractions = if let Some(by) = by {
            match by {
                ExtractionBy::Waypoint(symbol) => {
                    database::Extraction::get_by_waypoint_symbol(
                        &context.database_pool,
                        &symbol.to_string(),
                        query,
                    )
                    .await
                }
                ExtractionBy::System(symbol) => {
                    database::Extraction::get_by_system_symbol(
                        &context.database_pool,
                        &symbol.to_string(),
                        query,
                    )
                    .await
                }
                ExtractionBy::ShipSymbol(symbol) => {
                    database::Extraction::get_by_ship(
                        &context.database_pool,
                        &symbol.to_string(),
                        query,
                    )
                    .await
                }
                ExtractionBy::TradeSymbol(symbol) => {
                    database::Extraction::get_by_trade_symbol(
                        &context.database_pool,
                        &symbol,
                        query,
                    )
                    .await
                }
                ExtractionBy::Siphon(siphon) => {
                    database::Extraction::get_by_siphon(&context.database_pool, siphon, query).await
                }
                ExtractionBy::Survey(symbol) => {
                    database::Extraction::get_by_survey_symbol(
                        &context.database_pool,
                        &symbol.to_string(),
                        query,
                    )
                    .await
                }
            }
        } else {
            database::Extraction::get_all(&context.database_pool, query).await
        }?;
        Ok(extractions.into())
    }

    async fn fleets<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<FleetBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLFleetPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let fleets = if let Some(by) = by {
            match by {
                FleetBy::System(system_symbol) => {
                    database::Fleet::get_by_system(&context.database_pool, &system_symbol, query)
                        .await
                }
                FleetBy::Type(fleet_type) => {
                    database::Fleet::get_by_type(&context.database_pool, fleet_type, query).await
                }
            }
        } else {
            database::Fleet::get_all(&context.database_pool, query).await
        }?;
        Ok(fleets.into())
    }

    async fn ship_assignments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ShipAssignmentBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLShipAssignmentPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let ship_assignments = if let Some(by) = by {
            match by {
                ShipAssignmentBy::Fleet(fleet_id) => {
                    database::ShipAssignment::get_by_fleet_id(
                        &context.database_pool,
                        fleet_id,
                        query,
                    )
                    .await
                }
                ShipAssignmentBy::Open(_assigned) => {
                    database::ShipAssignment::get_open_assignments(&context.database_pool, query)
                        .await
                }
            }
        } else {
            database::ShipAssignment::get_all(&context.database_pool, query).await
        }?;
        Ok(ship_assignments.into())
    }

    async fn reserved_funds<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLReservedFundPage> {
        let context = ctx.data::<ConductorContext>()?;
        let reserved_funds = database::ReservedFund::get_all(
            &context.database_pool,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(reserved_funds.into())
    }

    async fn surveys<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<SurveyBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLSurveyPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let surveys = if let Some(by) = by {
            match by {
                SurveyBy::Waypoint(waypoint_symbol) => {
                    database::Survey::get_by_waypoint_symbol(
                        &context.database_pool,
                        &waypoint_symbol,
                        query,
                    )
                    .await
                }
                SurveyBy::System(system_symbol) => {
                    database::Survey::get_by_system_symbol(
                        &context.database_pool,
                        &system_symbol,
                        query,
                    )
                    .await
                }
                SurveyBy::Size(size) => {
                    database::Survey::get_by_size(&context.database_pool, size, query).await
                }
                SurveyBy::ShipSymbol(ship_symbol) => {
                    database::Survey::get_by_ship(&context.database_pool, &ship_symbol, query).await
                }
            }
        } else {
            database::Survey::get_all(&context.database_pool, query).await
        }?;
        Ok(surveys.into())
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLTradeRoutePage> {
        let context = ctx.data::<ConductorContext>()?;
        let trade_routes =
            database::TradeRoute::get_all(&context.database_pool, paginated_query(page, page_size))
                .await?;
        Ok(trade_routes.into())
    }

    async fn trade_route<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        route_id: i32,
    ) -> Result<gql_models::GQLTradeRoute> {
        let context = ctx.data::<ConductorContext>()?;
        let trade_route = database::TradeRoute::get_by_id(&context.database_pool, &route_id)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(trade_route.into())
    }

    async fn ship_infos<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLShipInfoPage> {
        let context = ctx.data::<ConductorContext>()?;
        let ship_info =
            database::ShipInfo::get_all(&context.database_pool, paginated_query(page, page_size))
                .await?;
        Ok(ship_info.into())
    }

    async fn ship_info<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        symbol: String,
    ) -> Result<gql_models::GQLShipInfo> {
        let context = ctx.data::<ConductorContext>()?;
        let ship_info = database::ShipInfo::get_by_id(&context.database_pool, &symbol)
            .await?
            .ok_or(GraphiQLError::NotFound)?;
        Ok(ship_info.into())
    }

    async fn ship_states<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ShipStateBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLShipStatePage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let ship_states = if let Some(by) = by {
            match by {
                ShipStateBy::Waypoint(waypoint) => {
                    database::ShipState::get_by_waypoint(&context.database_pool, &waypoint, query)
                        .await
                }
                ShipStateBy::System(system) => {
                    database::ShipState::get_by_system(&context.database_pool, &system, query).await
                }
                ShipStateBy::ShipSymbol(symbol) => {
                    database::ShipState::get_by_ship(&context.database_pool, &symbol, query).await
                }
            }
        } else {
            database::ShipState::get_all(&context.database_pool, query).await
        }?;
        Ok(ship_states.into())
    }

    async fn ship_events<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ShipEventBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLShipEventPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let ship_events = if let Some(by) = by {
            match by {
                ShipEventBy::ShipSymbol(symbol) => {
                    database::ShipEvent::get_by_ship(&context.database_pool, &symbol, query).await
                }
            }
        } else {
            database::ShipEvent::get_all(&context.database_pool, query).await
        }?;
        Ok(ship_events.into())
    }

    async fn shipyards<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLShipyardPage> {
        let context = ctx.data::<ConductorContext>()?;
        let shipyards = database::Shipyard::get_last_paginated(
            &context.database_pool,
            paginated_query(page, page_size),
        )
        .await?;
        Ok(shipyards.into())
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
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLShipyardShipPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let shipyard_ships = if let Some(by) = by {
            match by {
                ShipyardShipBy::Waypoint(waypoint) => {
                    database::ShipyardShip::get_last_by_waypoint(
                        &context.database_pool,
                        &waypoint,
                        query,
                    )
                    .await
                }
                ShipyardShipBy::System(system) => {
                    database::ShipyardShip::get_last_by_system(
                        &context.database_pool,
                        &system,
                        query,
                    )
                    .await
                }
                ShipyardShipBy::ShipSymbol(symbol) => {
                    database::ShipyardShip::get_last_by_ship_symbol(
                        &context.database_pool,
                        &symbol,
                        query,
                    )
                    .await
                }
            }
        } else {
            database::ShipyardShip::get_last_paginated(&context.database_pool, query).await
        }?;
        Ok(shipyard_ships.into())
    }

    async fn construction_materials<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ConstructionMaterialBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLConstructionMaterialPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let construction_materials = if let Some(by) = by {
            match by {
                ConstructionMaterialBy::Waypoint(waypoint) => {
                    database::ConstructionMaterial::get_by_waypoint(
                        &context.database_pool,
                        &waypoint,
                        query,
                    )
                    .await
                }
                ConstructionMaterialBy::System(system) => {
                    database::ConstructionMaterial::get_by_system(
                        &context.database_pool,
                        &system,
                        query,
                    )
                    .await
                }
                ConstructionMaterialBy::TradeSymbol(trade_symbol) => {
                    database::ConstructionMaterial::get_by_trade_symbol(
                        &context.database_pool,
                        &trade_symbol,
                        query,
                    )
                    .await
                }
            }
        } else {
            database::ConstructionMaterial::get_all(&context.database_pool, query).await
        }?;
        Ok(construction_materials.into())
    }

    async fn construction_shipments<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<ConstructionShipmentBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLConstructionShipmentPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let construction_shipments = if let Some(by) = by {
            match by {
                ConstructionShipmentBy::Waypoint(waypoint) => {
                    database::ConstructionShipment::get_by_destination_waypoint(
                        &context.database_pool,
                        &waypoint,
                        query,
                    )
                    .await
                }
                ConstructionShipmentBy::System(system) => {
                    database::ConstructionShipment::get_by_system(
                        &context.database_pool,
                        &system,
                        query,
                    )
                    .await
                }
                ConstructionShipmentBy::TradeSymbol(trade_symbol) => {
                    database::ConstructionShipment::get_by_trade_symbol(
                        &context.database_pool,
                        &trade_symbol,
                        query,
                    )
                    .await
                }
                ConstructionShipmentBy::Material(material_id) => {
                    database::ConstructionShipment::get_by_material_id(
                        &context.database_pool,
                        material_id,
                        query,
                    )
                    .await
                }
                ConstructionShipmentBy::ShipSymbol(ship_symbol) => {
                    database::ConstructionShipment::get_by_ship(
                        &context.database_pool,
                        &ship_symbol,
                        query,
                    )
                    .await
                }
            }
        } else {
            database::ConstructionShipment::get_all(&context.database_pool, query).await
        }?;
        Ok(construction_shipments.into())
    }

    async fn jump_gate_connections<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        from: Option<String>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLJumpGateConnectionPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let jump_gate_connections = if let Some(from) = from {
            database::JumpGateConnection::get_all_from(&context.database_pool, &from, query).await?
        } else {
            database::JumpGateConnection::get_all(&context.database_pool, query).await?
        };
        Ok(jump_gate_connections.into())
    }

    async fn jump_connections<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLGateConnPage> {
        let context = ctx.data::<ConductorContext>()?;
        let connections = database::JumpGateConnection::get_all(
            &context.database_pool,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items;

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

        let gate_waypoints = database::Waypoint::get_all(
            &context.database_pool,
            database::PaginatedQuery::unpaged(),
        )
        .await?
        .items
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

        Ok(database::paginate_items(
            paginated_query(page, page_size),
            connection_map.into_values().collect(),
        )?
        .into())
    }

    async fn market_trades<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<MarketTradeBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLMarketTradePage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let market_trades = if let Some(by) = by {
            match by {
                MarketTradeBy::Waypoint(waypoint) => {
                    database::MarketTrade::get_last_by_waypoint(
                        &context.database_pool,
                        &waypoint,
                        query,
                    )
                    .await
                }
                MarketTradeBy::TradeSymbol(trade_symbol) => {
                    database::MarketTrade::get_last_by_symbol(
                        &context.database_pool,
                        &trade_symbol,
                        query,
                    )
                    .await
                }
                MarketTradeBy::System(system) => {
                    database::MarketTrade::get_last_by_system(
                        &context.database_pool,
                        &system,
                        query,
                    )
                    .await
                }
            }
        } else {
            database::MarketTrade::get_last(&context.database_pool, query).await
        }?;
        Ok(market_trades.into())
    }

    async fn market_trade_goods<'ctx>(
        &self,
        ctx: &async_graphql::Context<'ctx>,
        by: Option<MarketTradeGoodBy>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<gql_models::GQLMarketTradeGoodPage> {
        let context = ctx.data::<ConductorContext>()?;
        let query = paginated_query(page, page_size);
        let market_trade_goods = if let Some(by) = by {
            match by {
                MarketTradeGoodBy::Waypoint(waypoint) => {
                    database::MarketTradeGood::get_last_by_waypoint(
                        &context.database_pool,
                        &waypoint,
                        query,
                    )
                    .await
                }
                MarketTradeGoodBy::TradeSymbol(trade_symbol) => {
                    database::MarketTradeGood::get_last_by_symbol(
                        &context.database_pool,
                        &trade_symbol,
                        query,
                    )
                    .await
                }
                MarketTradeGoodBy::System(system) => {
                    database::MarketTradeGood::get_last_by_system(
                        &context.database_pool,
                        &system,
                        query,
                    )
                    .await
                }
            }
        } else {
            database::MarketTradeGood::get_all(&context.database_pool, query).await
        }?;
        Ok(market_trade_goods.into())
    }

    async fn trade_symbol_infos(&self) -> Result<Vec<gql_models::TradeSymbolInfo>> {
        let trade_symbol_infos = models::TradeSymbol::iter().map(Into::into).collect();
        Ok(trade_symbol_infos)
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
enum ShipEventBy {
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
