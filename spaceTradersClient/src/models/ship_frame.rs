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

/// ShipFrame : The frame of the ship. The frame determines the number of modules and mounting points of the ship, as well as base fuel capacity. As the condition of the frame takes more wear, the ship will become more sluggish and less maneuverable.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipFrame {
    /// Symbol of the frame.
    #[serde(rename = "symbol")]
    pub symbol: Symbol,
    /// Name of the frame.
    #[serde(rename = "name")]
    pub name: String,
    /// Description of the frame.
    #[serde(rename = "description")]
    pub description: String,
    /// The repairable condition of a component. A value of 0 indicates the component needs significant repairs, while a value of 1 indicates the component is in near perfect condition. As the condition of a component is repaired, the overall integrity of the component decreases.
    #[serde(rename = "condition")]
    pub condition: f64,
    /// The overall integrity of the component, which determines the performance of the component. A value of 0 indicates that the component is almost completely degraded, while a value of 1 indicates that the component is in near perfect condition. The integrity of the component is non-repairable, and represents permanent wear over time.
    #[serde(rename = "integrity")]
    pub integrity: f64,
    /// The amount of slots that can be dedicated to modules installed in the ship. Each installed module take up a number of slots, and once there are no more slots, no new modules can be installed.
    #[serde(rename = "moduleSlots")]
    pub module_slots: i32,
    /// The amount of slots that can be dedicated to mounts installed in the ship. Each installed mount takes up a number of points, and once there are no more points remaining, no new mounts can be installed.
    #[serde(rename = "mountingPoints")]
    pub mounting_points: i32,
    /// The maximum amount of fuel that can be stored in this ship. When refueling, the ship will be refueled to this amount.
    #[serde(rename = "fuelCapacity")]
    pub fuel_capacity: i32,
    #[serde(rename = "requirements")]
    pub requirements: Box<models::ShipRequirements>,
}

impl ShipFrame {
    /// The frame of the ship. The frame determines the number of modules and mounting points of the ship, as well as base fuel capacity. As the condition of the frame takes more wear, the ship will become more sluggish and less maneuverable.
    pub fn new(symbol: Symbol, name: String, description: String, condition: f64, integrity: f64, module_slots: i32, mounting_points: i32, fuel_capacity: i32, requirements: models::ShipRequirements) -> ShipFrame {
        ShipFrame {
            symbol,
            name,
            description,
            condition,
            integrity,
            module_slots,
            mounting_points,
            fuel_capacity,
            requirements: Box::new(requirements),
        }
    }
}
/// Symbol of the frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Symbol {
    #[serde(rename = "FRAME_PROBE")]
    Probe,
    #[serde(rename = "FRAME_DRONE")]
    Drone,
    #[serde(rename = "FRAME_INTERCEPTOR")]
    Interceptor,
    #[serde(rename = "FRAME_RACER")]
    Racer,
    #[serde(rename = "FRAME_FIGHTER")]
    Fighter,
    #[serde(rename = "FRAME_FRIGATE")]
    Frigate,
    #[serde(rename = "FRAME_SHUTTLE")]
    Shuttle,
    #[serde(rename = "FRAME_EXPLORER")]
    Explorer,
    #[serde(rename = "FRAME_MINER")]
    Miner,
    #[serde(rename = "FRAME_LIGHT_FREIGHTER")]
    LightFreighter,
    #[serde(rename = "FRAME_HEAVY_FREIGHTER")]
    HeavyFreighter,
    #[serde(rename = "FRAME_TRANSPORT")]
    Transport,
    #[serde(rename = "FRAME_DESTROYER")]
    Destroyer,
    #[serde(rename = "FRAME_CRUISER")]
    Cruiser,
    #[serde(rename = "FRAME_CARRIER")]
    Carrier,
}

impl Default for Symbol {
    fn default() -> Symbol {
        Self::Probe
    }
}

