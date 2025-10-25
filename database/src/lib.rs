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
pub use extraction::Extraction;
pub use fleet::TradeMode;
pub use frame_info::FrameInfo;
pub use jump_gate_connection::JumpGateConnection;
pub use market_trade::MarketTrade;
pub use market_trade_good::MarketTradeGood;
pub use market_transaction::MarketTransaction;
pub use market_transaction::TransactionReason;
pub use module_info::ModuleInfo;
pub use mount_info::MountInfo;
pub use reactor_info::ReactorInfo;
pub use reserved_fund::FundStatus;
pub use reserved_fund::ReservedFund;
pub use route::Route;
pub use ship_info::ShipInfo;
pub use ship_jump::ShipJump;
pub use ship_state::ShipState;
pub use shipyard::Shipyard;
pub use shipyard_ship::ShipyardShip;
pub use shipyard_ship_types::ShipyardShipTypes;
pub use shipyard_transaction::ShipyardTransaction;
pub use survey::Survey;
pub use system::RespSystem;
pub use system::System;
pub use trade_route::TradeRoute;
pub use waypoint::Waypoint;

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

#[allow(async_fn_in_trait)]
pub trait DatabaseConnector<T> {
    /// Insert a new item into the database, or update it if it already exists.
    async fn insert(database_pool: &DbPool, item: &T) -> crate::Result<()>;
    /// Insert multiple items into the database, or update them if they already exist.
    async fn insert_bulk(database_pool: &DbPool, items: &[T]) -> crate::Result<()>;
    #[allow(dead_code)]
    /// Get all items from the database.
    async fn get_all(database_pool: &DbPool) -> crate::Result<Vec<T>>;
}
