/*
 * SpaceTraders API
 *
 * SpaceTraders is an open-universe game and learning platform that offers a set of HTTP endpoints to control a fleet of ships and explore a multiplayer universe.  The API is documented using [OpenAPI](https://github.com/SpaceTradersAPI/api-docs). You can send your first request right here in your browser to check the status of the game server.  ```json http {   \"method\": \"GET\",   \"url\": \"https://api.spacetraders.io/v2\", } ```  Unlike a traditional game, SpaceTraders does not have a first-party client or app to play the game. Instead, you can use the API to build your own client, write a script to automate your ships, or try an app built by the community.  We have a [Discord channel](https://discord.com/invite/jh6zurdWk5) where you can share your projects, ask questions, and get help from other players.
 *
 * The version of the OpenAPI document: 2.3.0
 * Contact: joel@spacetraders.io
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// Market : Market details.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Market {
    /// The symbol of the market. The symbol is the same as the waypoint where the market is located.
    #[serde(rename = "symbol")]
    pub symbol: String,
    /// The list of goods that are exported from this market.
    #[serde(rename = "exports")]
    pub exports: Vec<models::TradeGood>,
    /// The list of goods that are sought as imports in this market.
    #[serde(rename = "imports")]
    pub imports: Vec<models::TradeGood>,
    /// The list of goods that are bought and sold between agents at this market.
    #[serde(rename = "exchange")]
    pub exchange: Vec<models::TradeGood>,
    /// The list of recent transactions at this market. Visible only when a ship is present at the market.
    #[serde(rename = "transactions", skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<models::MarketTransaction>>,
    /// The list of goods that are traded at this market. Visible only when a ship is present at the market.
    #[serde(rename = "tradeGoods", skip_serializing_if = "Option::is_none")]
    pub trade_goods: Option<Vec<models::MarketTradeGood>>,
}

impl Market {
    /// Market details.
    pub fn new(
        symbol: String,
        exports: Vec<models::TradeGood>,
        imports: Vec<models::TradeGood>,
        exchange: Vec<models::TradeGood>,
    ) -> Market {
        Market {
            symbol,
            exports,
            imports,
            exchange,
            transactions: None,
            trade_goods: None,
        }
    }
}
