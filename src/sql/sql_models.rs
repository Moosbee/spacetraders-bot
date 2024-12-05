use space_traders_client::models;

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

#[derive(Clone, Debug, PartialEq)]
pub struct Waypoint {
    pub symbol: String,
    pub system_symbol: String,
    pub created_at: sqlx::types::time::PrimitiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, PartialEq, Eq)]
pub struct MarketTradeGood {
    pub symbol: models::TradeSymbol,
    pub waypoint_symbol: String,
    pub r#type: models::market_trade_good::Type,
    pub trade_volume: i32,
    pub supply: models::SupplyLevel,
    pub activity: Option<models::ActivityLevel>,
    pub purchase_price: i32,
    pub sell_price: i32,
    pub created: sqlx::types::time::PrimitiveDateTime,
    pub created_at: sqlx::types::time::PrimitiveDateTime,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct MarketTransaction {
    /// The symbol of the waypoint.
    pub waypoint_symbol: String,
    /// The symbol of the ship that made the transaction.
    pub ship_symbol: String,
    /// The symbol of the trade good.
    pub trade_symbol: models::TradeSymbol,
    /// The type of transaction.
    pub r#type: models::market_transaction::Type,
    /// The number of units of the transaction.
    pub units: i32,
    /// The price per unit of the transaction.
    pub price_per_unit: i32,
    /// The total price of the transaction.
    pub total_price: i32,
    /// The timestamp of the transaction.
    pub timestamp: String,
    /// The reason for the transaction.
    /// pub reason: TransactionReason,
    pub contract: Option<String>,
    pub trade_route: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TransactionReason {
    Contract(String),
    TradeRoute(i32),
    #[default]
    None,
}

#[derive(Debug, Clone, sqlx::FromRow, PartialEq, Eq)]
pub struct MarketTrade {
    pub waypoint_symbol: String,
    pub symbol: models::TradeSymbol,
    pub created_at: sqlx::types::time::PrimitiveDateTime,
    pub r#type: models::market_trade_good::Type,
}

impl Default for MarketTrade {
    fn default() -> MarketTrade {
        MarketTrade {
            waypoint_symbol: String::new(),
            symbol: models::TradeSymbol::PreciousStones,
            created_at: sqlx::types::time::PrimitiveDateTime::MIN,
            r#type: models::market_trade_good::Type::Exchange,
        }
    }
}

pub struct Contract {
    pub id: String,
    pub faction_symbol: String,
    pub contract_type: models::contract::Type,
    pub accepted: bool,
    pub fulfilled: bool,
    pub deadline_to_accept: Option<String>,
    pub on_accepted: i32,
    pub on_fulfilled: i32,
    pub deadline: String,
}

pub struct ContractDelivery {
    pub contract_id: String,
    pub trade_symbol: models::TradeSymbol,
    pub destination_symbol: String,
    pub units_required: i32,
    pub units_fulfilled: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Agent {
    pub symbol: String,
    pub account_id: Option<String>,
    pub headquarters: String,
    pub credits: i64,
    pub starting_faction: String,
    pub ship_count: i32,
    #[allow(dead_code)]
    #[serde(skip)]
    pub created_at: sqlx::types::time::PrimitiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeRoute {
    pub id: i32,
    pub symbol: models::TradeSymbol,
    pub ship_symbol: String,
    pub purchase_waypoint: String,
    pub sell_waypoint: String,
    pub finished: bool,
    pub trade_volume: i32,
    pub predicted_purchase_price: i32,
    pub predicted_sell_price: i32,
    pub created_at: sqlx::types::time::PrimitiveDateTime,
}

impl TradeRoute {
    pub fn complete(self) -> Self {
        TradeRoute {
            finished: true,
            ..self
        }
    }
}

impl Default for TradeRoute {
    fn default() -> TradeRoute {
        TradeRoute {
            id: 0,
            symbol: models::TradeSymbol::PreciousStones,
            ship_symbol: String::new(),
            purchase_waypoint: String::new(),
            sell_waypoint: String::new(),
            finished: false,
            trade_volume: 0,
            predicted_purchase_price: 0,
            predicted_sell_price: 0,
            created_at: sqlx::types::time::PrimitiveDateTime::MIN,
        }
    }
}

impl std::fmt::Display for TradeRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}: {} -> {} {}",
            self.ship_symbol,
            self.symbol,
            self.purchase_waypoint,
            self.sell_waypoint,
            self.trade_volume * self.predicted_sell_price
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub id: i32,
    pub from: String,
    pub to: String,
    pub nav_mode: String,
    pub speed: i32,
    pub fuel_cost: i32,
    pub travel_time: f64,
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
