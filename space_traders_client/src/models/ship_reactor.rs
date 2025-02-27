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

/// ShipReactor : The reactor of the ship. The reactor is responsible for powering the ship's systems and weapons.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipReactor {
    /// Symbol of the reactor.
    #[serde(rename = "symbol")]
    pub symbol: Symbol,
    /// Name of the reactor.
    #[serde(rename = "name")]
    pub name: String,
    /// Description of the reactor.
    #[serde(rename = "description")]
    pub description: String,
    /// The repairable condition of a component. A value of 0 indicates the component needs significant repairs, while a value of 1 indicates the component is in near perfect condition. As the condition of a component is repaired, the overall integrity of the component decreases.
    #[serde(rename = "condition")]
    pub condition: f64,
    /// The overall integrity of the component, which determines the performance of the component. A value of 0 indicates that the component is almost completely degraded, while a value of 1 indicates that the component is in near perfect condition. The integrity of the component is non-repairable, and represents permanent wear over time.
    #[serde(rename = "integrity")]
    pub integrity: f64,
    /// The amount of power provided by this reactor. The more power a reactor provides to the ship, the lower the cooldown it gets when using a module or mount that taxes the ship's power.
    #[serde(rename = "powerOutput")]
    pub power_output: i32,
    #[serde(rename = "requirements")]
    pub requirements: Box<models::ShipRequirements>,
    /// The overall quality of the component, which determines the quality of the component. High quality components return more ships parts and ship plating when a ship is scrapped. But also require more of these parts to repair. This is transparent to the player, as the parts are bought from/sold to the marketplace.
    #[serde(rename = "quality")]
    pub quality: f64,
}

impl ShipReactor {
    /// The reactor of the ship. The reactor is responsible for powering the ship's systems and weapons.
    pub fn new(
        symbol: Symbol,
        name: String,
        description: String,
        condition: f64,
        integrity: f64,
        power_output: i32,
        requirements: models::ShipRequirements,
        quality: f64,
    ) -> ShipReactor {
        ShipReactor {
            symbol,
            name,
            description,
            condition,
            integrity,
            power_output,
            requirements: Box::new(requirements),
            quality,
        }
    }
}
/// Symbol of the reactor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Symbol {
    #[serde(rename = "REACTOR_SOLAR_I")]
    SolarI,
    #[serde(rename = "REACTOR_FUSION_I")]
    FusionI,
    #[serde(rename = "REACTOR_FISSION_I")]
    FissionI,
    #[serde(rename = "REACTOR_CHEMICAL_I")]
    ChemicalI,
    #[serde(rename = "REACTOR_ANTIMATTER_I")]
    AntimatterI,
}

impl Default for Symbol {
    fn default() -> Symbol {
        Self::SolarI
    }
}
