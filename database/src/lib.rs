use std::future::Future;

mod agent;
mod construction_material;
mod construction_shipment;
mod contract;
mod contract_delivery;
mod contract_shipment;
mod market_trade;
mod market_trade_good;
mod route;
mod ship_info;
mod system;
mod trade_route;
mod waypoint;

mod shipyard;
mod shipyard_ship;
mod shipyard_ship_types;

mod chart_transaction; // + credits
mod market_transaction; // + credits - credits
mod repair_transaction; // - credits
mod scrap_transaction; // + credits
mod ship_modification_transaction; // - credits
mod shipyard_transaction; // - credits

mod engine_info;
mod frame_info;
mod module_info;
mod mount_info;
mod reactor_info;

mod export_import;
mod extraction;
mod fleet;
mod jump_gate_connection;
mod reserved_fund;
mod ship_assignment;
mod ship_jump;
mod ship_state;
mod survey;

pub use agent::Agent;
pub use chart_transaction::ChartTransaction;
pub use construction_material::ConstructionMaterial;
pub use construction_shipment::ConstructionShipment;
pub use contract::Contract;
pub use contract_delivery::ContractDelivery;
pub use contract_shipment::ContractShipment;
pub use contract_shipment::ShipmentStatus;
pub use engine_info::EngineInfo;
pub use export_import::ExportImportMapping;
pub use extraction::Extraction;
pub use fleet::ChartingConfig as ChartingFleetConfig;
pub use fleet::ConstructionConfig as ConstructionFleetConfig;
pub use fleet::ContractConfig as ContractFleetConfig;
pub use fleet::Fleet;
pub use fleet::FleetBySystemLoader;
pub use fleet::FleetConfig;
pub use fleet::FleetLoader;
pub use fleet::FleetType;
pub use fleet::ManuelConfig as ManuelFleetConfig;
pub use fleet::MiningConfig as MiningFleetConfig;
pub use fleet::ScrapingConfig as ScrapingFleetConfig;
pub use fleet::TradeMode;
pub use fleet::TradingConfig as TradingFleetConfig;
pub use frame_info::FrameInfo;
pub use jump_gate_connection::JumpGateConnection;
pub use market_trade::MarketTrade;
pub use market_trade_good::MarketTradeGood;
pub use market_transaction::MarketTransaction;
pub use market_transaction::TransactionReason;
pub use market_transaction::TransactionSummary;
pub use module_info::ModuleInfo;
pub use mount_info::MountInfo;
pub use reactor_info::ReactorInfo;
pub use repair_transaction::RepairTransaction;
pub use reserved_fund::FundStatus;
pub use reserved_fund::ReservedFund;
pub use route::Route;
pub use scrap_transaction::ScrapTransaction;
pub use ship_assignment::AssignmentsByFleetLoader;
pub use ship_assignment::ShipAssignment;
pub use ship_assignment::SimpleShipRequirement;
pub use ship_info::ShipInfo;
pub use ship_jump::ShipJump;
pub use ship_modification_transaction::ShipModificationTransaction;
pub use ship_state::ShipState;
pub use shipyard::Shipyard;
pub use shipyard_ship::ShipyardShip;
pub use shipyard_ship_types::ShipyardShipTypes;
pub use shipyard_transaction::ShipyardTransaction;
pub use survey::Survey;
pub use system::System;
pub use trade_route::TradeRoute;
pub use waypoint::Waypoint;
pub use waypoint::WaypointLoader;
pub use waypoint::WaypointSystemLoader;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("Invalid ship info role: {0}")]
    InvalidShipInfoRole(String),

    #[error("Invalid trade symbol: {0}")]
    InvalidTradeSymbol(String),

    #[error(transparent)]
    Chrono(#[from] chrono::ParseError),

    #[error("Invalid ship type: {0}")]
    InvalidShipType(String),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Invalid pagination query: page={page}, page_size={page_size:?}")]
    InvalidPaginationQuery { page: i64, page_size: Option<i64> },

    #[error("Incomplete fleet config for fleet ID {fleet_id:?}")]
    IncompleteFleetConfig { fleet_id: i32 },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct DbPool {
    pub database_pool: sqlx::PgPool,
    pub readyset_pool: Option<sqlx::PgPool>,
    pub agent_broadcast_channel: (
        tokio::sync::broadcast::Sender<Agent>,
        tokio::sync::broadcast::Receiver<Agent>,
    ),
}

impl DbPool {
    pub fn new(database_pool: sqlx::PgPool, readyset_pool: Option<sqlx::PgPool>) -> DbPool {
        let agent_broadcast_channel = tokio::sync::broadcast::channel(10);
        DbPool {
            database_pool,
            readyset_pool,
            agent_broadcast_channel,
        }
    }
    pub fn get_cache_pool(&self) -> &sqlx::PgPool {
        if let Some(pool) = &self.readyset_pool {
            pool
        } else {
            &self.database_pool
        }
    }
}

impl Clone for DbPool {
    fn clone(&self) -> Self {
        Self {
            database_pool: self.database_pool.clone(),
            readyset_pool: self.readyset_pool.clone(),
            agent_broadcast_channel: (
                self.agent_broadcast_channel.0.clone(),
                self.agent_broadcast_channel.0.subscribe(),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginatedQuery {
    pub page: i64,
    pub page_size: Option<i64>,
}

impl PaginatedQuery {
    pub const fn unpaged() -> Self {
        Self {
            page: 1,
            page_size: None,
        }
    }

    pub const fn new(page: i64, page_size: Option<i64>) -> Self {
        Self { page, page_size }
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.page < 1 {
            return Err(crate::Error::InvalidPaginationQuery {
                page: self.page,
                page_size: self.page_size,
            });
        }

        if matches!(self.page_size, Some(page_size) if page_size < 1) {
            return Err(crate::Error::InvalidPaginationQuery {
                page: self.page,
                page_size: self.page_size,
            });
        }

        Ok(())
    }

    pub fn validated(self) -> crate::Result<Self> {
        self.validate()?;
        Ok(self)
    }

    pub const fn is_unpaged(&self) -> bool {
        self.page_size.is_none()
    }

    pub fn offset(&self) -> crate::Result<i64> {
        self.validate()?;

        Ok(match self.page_size {
            Some(page_size) => (self.page - 1) * page_size,
            None => 0,
        })
    }
}

impl Default for PaginatedQuery {
    fn default() -> Self {
        Self::unpaged()
    }
}

pub async fn run_paginated_query<
    T,
    FetchPage,
    PageFuture,
    FetchAll,
    AllFuture,
    FetchCount,
    CountFuture,
>(
    query: PaginatedQuery,
    fetch_page: FetchPage,
    fetch_all: FetchAll,
    fetch_count: FetchCount,
) -> crate::Result<PaginatedResult<T>>
where
    FetchPage: FnOnce(i64, i64) -> PageFuture,
    PageFuture: Future<Output = crate::Result<Vec<T>>>,
    FetchAll: FnOnce() -> AllFuture,
    AllFuture: Future<Output = crate::Result<Vec<T>>>,
    FetchCount: FnOnce() -> CountFuture,
    CountFuture: Future<Output = crate::Result<i64>>,
{
    let query = query.validated()?;

    let items = if let Some(page_size) = query.page_size {
        fetch_page(page_size, query.offset()?).await?
    } else {
        fetch_all().await?
    };

    let total_count = if query.is_unpaged() {
        items.len() as i64
    } else {
        fetch_count().await?
    };

    Ok(PaginatedResult {
        items,
        total_count,
        page: query.page,
        page_size: query.page_size,
    })
}

pub fn paginate_items<T>(
    query: PaginatedQuery,
    items: Vec<T>,
) -> crate::Result<PaginatedResult<T>> {
    let query = query.validated()?;
    let total_count = items.len() as i64;

    let items = if let Some(page_size) = query.page_size {
        items
            .into_iter()
            .skip(query.offset()? as usize)
            .take(page_size as usize)
            .collect()
    } else {
        items
    };

    Ok(PaginatedResult {
        items,
        total_count,
        page: query.page,
        page_size: query.page_size,
    })
}

#[allow(async_fn_in_trait)]
pub trait DatabaseConnectorAsync: std::marker::Sized {
    type ID;
    async fn insert_new(database_pool: &DbPool, item: &Self) -> crate::Result<Self::ID>;
    async fn upsert(database_pool: &DbPool, item: &Self) -> crate::Result<()>;
    async fn update(database_pool: &DbPool, item: &Self) -> crate::Result<()>;
    async fn insert_bulk(database_pool: &DbPool, items: &[Self]) -> crate::Result<()>;
    async fn get_all(
        database_pool: &DbPool,
        query: PaginatedQuery,
    ) -> crate::Result<PaginatedResult<Self>>;
    async fn get_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<Option<Self>>;
    async fn delete_by_id(database_pool: &DbPool, id: &Self::ID) -> crate::Result<()>;
    fn set_id(&mut self, id: Self::ID);
}
