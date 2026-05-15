use std::fmt::{Display, Formatter};

use chrono::Utc;
use space_traders_client::models;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct ShipEventBroadcaster {
    pub sender: broadcast::Sender<database::ShipEvent>,
    pub receiver: broadcast::Receiver<database::ShipEvent>,
}

impl Default for ShipEventBroadcaster {
    fn default() -> Self {
        let (sender, receiver) = broadcast::channel(256);
        Self { sender, receiver }
    }
}

impl Clone for ShipEventBroadcaster {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.resubscribe(),
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipEventKind {
    Autopilot,
    Cargo,
    Mining,
}

impl ShipEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ShipEventKind::Autopilot => "AUTOPILOT",
            ShipEventKind::Cargo => "CARGO",
            ShipEventKind::Mining => "MINING",
        }
    }
}

impl Display for ShipEventKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipEventName {
    JumpConnection,
    WarpConnection,
    NavigateConnection,
    PurchaseCargo,
    SellCargo,
    ExtractResources,
    ExtractResourcesWithSurvey,
    SiphonResources,
    CreateSurvey,
}

impl ShipEventName {
    pub fn as_str(self) -> &'static str {
        match self {
            ShipEventName::JumpConnection => "JUMP_CONNECTION",
            ShipEventName::WarpConnection => "WARP_CONNECTION",
            ShipEventName::NavigateConnection => "NAVIGATE_CONNECTION",
            ShipEventName::PurchaseCargo => "PURCHASE_CARGO",
            ShipEventName::SellCargo => "SELL_CARGO",
            ShipEventName::ExtractResources => "EXTRACT_RESOURCES",
            ShipEventName::ExtractResourcesWithSurvey => "EXTRACT_RESOURCES_WITH_SURVEY",
            ShipEventName::SiphonResources => "SIPHON_RESOURCES",
            ShipEventName::CreateSurvey => "CREATE_SURVEY",
        }
    }
}

impl Display for ShipEventName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipEventPhase {
    Completed,
}

impl ShipEventPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            ShipEventPhase::Completed => "COMPLETED",
        }
    }
}

impl Display for ShipEventPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShipEventRecord<P> {
    pub kind: ShipEventKind,
    pub name: ShipEventName,
    pub phase: ShipEventPhase,
    pub correlation_id: String,
    pub payload: P,
}

impl<P> ShipEventRecord<P> {
    pub fn autopilot_connection(
        ship_symbol: &str,
        event_name: ShipEventName,
        from: &str,
        to: &str,
        payload: P,
    ) -> Self {
        Self {
            kind: ShipEventKind::Autopilot,
            name: event_name,
            phase: ShipEventPhase::Completed,
            correlation_id: format!(
                "{}:{}:{}:{}:{}",
                ship_symbol,
                event_name.as_str(),
                from,
                to,
                Utc::now().timestamp_millis()
            ),
            payload,
        }
    }

    pub fn cargo_trade(
        ship_symbol: &str,
        event_name: ShipEventName,
        waypoint_symbol: &str,
        trade_symbol: &models::TradeSymbol,
        payload: P,
    ) -> Self {
        Self {
            kind: ShipEventKind::Cargo,
            name: event_name,
            phase: ShipEventPhase::Completed,
            correlation_id: format!(
                "{}:{}:{}:{}:{}",
                ship_symbol,
                event_name.as_str(),
                waypoint_symbol,
                trade_symbol,
                Utc::now().timestamp_millis()
            ),
            payload,
        }
    }

    pub fn mining_action(
        ship_symbol: &str,
        event_name: ShipEventName,
        waypoint_symbol: &str,
        payload: P,
    ) -> Self {
        Self {
            kind: ShipEventKind::Mining,
            name: event_name,
            phase: ShipEventPhase::Completed,
            correlation_id: format!(
                "{}:{}:{}:{}",
                ship_symbol,
                event_name.as_str(),
                waypoint_symbol,
                Utc::now().timestamp_millis()
            ),
            payload,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JumpConnectionCompletedEvent {
    pub from: String,
    pub to: String,
    pub distance: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WarpConnectionCompletedEvent {
    pub from: String,
    pub to: String,
    pub nav_mode: String,
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NavigateConnectionCompletedEvent {
    pub from: String,
    pub to: String,
    pub nav_mode: String,
    pub distance: f64,
    pub fuel_cost: i32,
    pub travel_time: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CargoTradeCompletedEvent {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MiningExtractionCompletedEvent {
    pub waypoint_symbol: String,
    pub siphon: bool,
    pub yield_symbol: models::TradeSymbol,
    pub yield_units: i32,
    pub survey_signature: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MiningSurveyCreatedEvent {
    pub waypoint_symbol: String,
    pub surveys_created: i32,
    pub survey_signatures: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ShipEventPayload {
    JumpConnectionCompleted(JumpConnectionCompletedEvent),
    WarpConnectionCompleted(WarpConnectionCompletedEvent),
    NavigateConnectionCompleted(NavigateConnectionCompletedEvent),
    CargoTradeCompleted(CargoTradeCompletedEvent),
    MiningExtractionCompleted(MiningExtractionCompletedEvent),
    MiningSurveyCreated(MiningSurveyCreatedEvent),
}
