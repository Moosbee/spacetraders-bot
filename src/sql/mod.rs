mod agent;
mod construction_material;
mod construction_shipment;
mod contract;
mod contract_delivery;
mod contract_shipment;
mod market_trade;
mod market_trade_good;
mod market_transaction;
mod route;
mod ship_info;
mod system;
mod trade_route;
mod waypoint;

mod shipyard;
mod shipyard_ship;
mod shipyard_ship_types;
mod shipyard_transaction;

mod repair_transaction;
mod scrap_transaction;
mod ship_modification_transaction;

mod engine_info;
mod frame_info;
mod module_info;
mod mount_info;
mod reactor_info;

mod extraction;
mod ship_state;

pub use agent::Agent;
pub use construction_material::ConstructionMaterial;
pub use construction_shipment::ConstructionShipment;
pub use contract::Contract;
pub use contract_delivery::ContractDelivery;
pub use contract_shipment::ContractShipment;
pub use contract_shipment::ShipmentStatus;
pub use engine_info::EngineInfo;
pub use extraction::Extraction;
pub use frame_info::FrameInfo;
pub use market_trade::MarketTrade;
pub use market_trade_good::MarketTradeGood;
pub use market_transaction::MarketTransaction;
pub use market_transaction::TransactionReason;
pub use module_info::ModuleInfo;
pub use mount_info::MountInfo;
pub use reactor_info::ReactorInfo;
pub use route::Route;
pub use ship_info::ShipInfo;
pub use ship_info::ShipInfoRole;
pub use ship_state::ShipState;
pub use shipyard::Shipyard;
pub use shipyard_ship::ShipyardShip;
pub use shipyard_ship_types::ShipyardShipTypes;
pub use shipyard_transaction::ShipyardTransaction;
pub use system::RespSystem;
pub use system::System;
pub use trade_route::TradeRoute;
pub use waypoint::Waypoint;

#[derive(Debug)]
pub struct DbPool {
    pub database_pool: sqlx::PgPool,
    pub agent_broadcast_channel: (
        tokio::sync::broadcast::Sender<Agent>,
        tokio::sync::broadcast::Receiver<Agent>,
    ),
}

impl DbPool {
    pub fn new(database_pool: sqlx::PgPool) -> DbPool {
        let agent_broadcast_channel = tokio::sync::broadcast::channel(10);
        DbPool {
            database_pool,
            agent_broadcast_channel,
        }
    }
}

impl Clone for DbPool {
    fn clone(&self) -> Self {
        Self {
            database_pool: self.database_pool.clone(),
            agent_broadcast_channel: (
                self.agent_broadcast_channel.0.clone(),
                self.agent_broadcast_channel.0.subscribe(),
            ),
        }
    }
}

pub trait DatabaseConnector<T> {
    /// Insert a new item into the database, or update it if it already exists.
    async fn insert(database_pool: &DbPool, item: &T) -> sqlx::Result<()>;
    /// Insert multiple items into the database, or update them if they already exist.
    async fn insert_bulk(database_pool: &DbPool, items: &Vec<T>) -> sqlx::Result<()>;
    #[allow(dead_code)]
    /// Get all items from the database.
    async fn get_all(database_pool: &DbPool) -> sqlx::Result<Vec<T>>;
}
