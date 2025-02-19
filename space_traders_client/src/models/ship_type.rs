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
use strum_macros::EnumString;

/// ShipType : Type of ship
/// Type of ship
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, EnumString,
)]
pub enum ShipType {
    #[serde(rename = "SHIP_PROBE")]
    #[strum(serialize = "SHIP_PROBE")]
    Probe,
    #[serde(rename = "SHIP_MINING_DRONE")]
    #[strum(serialize = "SHIP_MINING_DRONE")]
    MiningDrone,
    #[serde(rename = "SHIP_SIPHON_DRONE")]
    #[strum(serialize = "SHIP_SIPHON_DRONE")]
    SiphonDrone,
    #[serde(rename = "SHIP_INTERCEPTOR")]
    #[strum(serialize = "SHIP_INTERCEPTOR")]
    Interceptor,
    #[serde(rename = "SHIP_LIGHT_HAULER")]
    #[strum(serialize = "SHIP_LIGHT_HAULER")]
    LightHauler,
    #[serde(rename = "SHIP_COMMAND_FRIGATE")]
    #[strum(serialize = "SHIP_COMMAND_FRIGATE")]
    CommandFrigate,
    #[serde(rename = "SHIP_EXPLORER")]
    #[strum(serialize = "SHIP_EXPLORER")]
    Explorer,
    #[serde(rename = "SHIP_HEAVY_FREIGHTER")]
    #[strum(serialize = "SHIP_HEAVY_FREIGHTER")]
    HeavyFreighter,
    #[serde(rename = "SHIP_LIGHT_SHUTTLE")]
    #[strum(serialize = "SHIP_LIGHT_SHUTTLE")]
    LightShuttle,
    #[serde(rename = "SHIP_ORE_HOUND")]
    #[strum(serialize = "SHIP_ORE_HOUND")]
    OreHound,
    #[serde(rename = "SHIP_REFINING_FREIGHTER")]
    #[strum(serialize = "SHIP_REFINING_FREIGHTER")]
    RefiningFreighter,
    #[serde(rename = "SHIP_SURVEYOR")]
    #[strum(serialize = "SHIP_SURVEYOR")]
    Surveyor,
}

impl std::fmt::Display for ShipType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Probe => write!(f, "SHIP_PROBE"),
            Self::MiningDrone => write!(f, "SHIP_MINING_DRONE"),
            Self::SiphonDrone => write!(f, "SHIP_SIPHON_DRONE"),
            Self::Interceptor => write!(f, "SHIP_INTERCEPTOR"),
            Self::LightHauler => write!(f, "SHIP_LIGHT_HAULER"),
            Self::CommandFrigate => write!(f, "SHIP_COMMAND_FRIGATE"),
            Self::Explorer => write!(f, "SHIP_EXPLORER"),
            Self::HeavyFreighter => write!(f, "SHIP_HEAVY_FREIGHTER"),
            Self::LightShuttle => write!(f, "SHIP_LIGHT_SHUTTLE"),
            Self::OreHound => write!(f, "SHIP_ORE_HOUND"),
            Self::RefiningFreighter => write!(f, "SHIP_REFINING_FREIGHTER"),
            Self::Surveyor => write!(f, "SHIP_SURVEYOR"),
        }
    }
}

impl Default for ShipType {
    fn default() -> ShipType {
        Self::Probe
    }
}
