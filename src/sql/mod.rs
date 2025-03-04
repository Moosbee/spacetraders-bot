mod agent;
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

pub use agent::Agent;
pub use contract::Contract;
pub use contract_delivery::ContractDelivery;
pub use contract_shipment::ContractShipment;
pub use contract_shipment::ShipmentStatus;
pub use market_trade::MarketTrade;
pub use market_trade_good::MarketTradeGood;
pub use market_transaction::MarketTransaction;
pub use market_transaction::TransactionReason;
pub use route::Route;
pub use ship_info::ShipInfo;
pub use ship_info::ShipInfoRole;
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
