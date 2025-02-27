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

/// ShipNavFlightMode : The ship's set speed when traveling between waypoints or systems.
/// The ship's set speed when traveling between waypoints or systems.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ShipNavFlightMode {
    #[serde(rename = "DRIFT")]
    Drift,
    #[serde(rename = "STEALTH")]
    Stealth,
    #[serde(rename = "CRUISE")]
    Cruise,
    #[serde(rename = "BURN")]
    Burn,
}

impl std::fmt::Display for ShipNavFlightMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Drift => write!(f, "DRIFT"),
            Self::Stealth => write!(f, "STEALTH"),
            Self::Cruise => write!(f, "CRUISE"),
            Self::Burn => write!(f, "BURN"),
        }
    }
}

impl Default for ShipNavFlightMode {
    fn default() -> ShipNavFlightMode {
        Self::Drift
    }
}
