/*
 * SpaceTraders API
 *
 * SpaceTraders is an open-universe game and learning platform that offers a set of HTTP endpoints to control a fleet of ships and explore a multiplayer universe.  The API is documented using [OpenAPI](https://github.com/SpaceTradersAPI/api-docs). You can send your first request right here in your browser to check the status of the game server.  ```json http {   \"method\": \"GET\",   \"url\": \"https://api.spacetraders.io/v2\", } ```  Unlike a traditional game, SpaceTraders does not have a first-party client or app to play the game. Instead, you can use the API to build your own client, write a script to automate your ships, or try an app built by the community.  We have a [Discord channel](https://discord.com/invite/jh6zurdWk5) where you can share your projects, ask questions, and get help from other players.
 *
 * The version of the OpenAPI document: 2.0.0
 * Contact: joel@spacetraders.io
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketTradeGood {
    #[serde(rename = "symbol")]
    pub symbol: models::TradeSymbol,
    /// The type of trade good (export, import, or exchange).
    #[serde(rename = "type")]
    pub r#type: Type,
    /// This is the maximum number of units that can be purchased or sold at this market in a single trade for this good. Trade volume also gives an indication of price volatility. A market with a low trade volume will have large price swings, while high trade volume will be more resilient to price changes.
    #[serde(rename = "tradeVolume")]
    pub trade_volume: i32,
    #[serde(rename = "supply")]
    pub supply: models::SupplyLevel,
    #[serde(rename = "activity", skip_serializing_if = "Option::is_none")]
    pub activity: Option<models::ActivityLevel>,
    /// The price at which this good can be purchased from the market.
    #[serde(rename = "purchasePrice")]
    pub purchase_price: i32,
    /// The price at which this good can be sold to the market.
    #[serde(rename = "sellPrice")]
    pub sell_price: i32,
}

impl MarketTradeGood {
    pub fn new(
        symbol: models::TradeSymbol,
        r#type: Type,
        trade_volume: i32,
        supply: models::SupplyLevel,
        purchase_price: i32,
        sell_price: i32,
    ) -> MarketTradeGood {
        MarketTradeGood {
            symbol,
            r#type,
            trade_volume,
            supply,
            activity: None,
            purchase_price,
            sell_price,
        }
    }
}
/// The type of trade good (export, import, or exchange).
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "market_trade_good_type")]
pub enum Type {
    #[serde(rename = "EXPORT")]
    #[sqlx(rename = "EXPORT")]
    Export,
    #[serde(rename = "IMPORT")]
    #[sqlx(rename = "IMPORT")]
    Import,
    #[serde(rename = "EXCHANGE")]
    #[sqlx(rename = "EXCHANGE")]
    Exchange,
}

impl Default for Type {
    fn default() -> Type {
        Self::Export
    }
}
