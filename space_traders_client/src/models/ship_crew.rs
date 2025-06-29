/*
 * SpaceTraders API
 *
 * SpaceTraders is an open-universe game and learning platform that offers a set of HTTP endpoints to control a fleet of ships and explore a multiplayer universe.  The API is documented using [OpenAPI](https://github.com/SpaceTradersAPI/api-docs). You can send your first request right here in your browser to check the status of the game server.  ```json http {   \"method\": \"GET\",   \"url\": \"https://api.spacetraders.io/v2\", } ```  Unlike a traditional game, SpaceTraders does not have a first-party client or app to play the game. Instead, you can use the API to build your own client, write a script to automate your ships, or try an app built by the community.  We have a [Discord channel](https://discord.com/invite/jh6zurdWk5) where you can share your projects, ask questions, and get help from other players.
 *
 * The version of the OpenAPI document: v2.3.0
 * Contact: joel@spacetraders.io
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ShipCrew : The ship's crew service and maintain the ship's systems and equipment.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipCrew {
    /// The current number of crew members on the ship.
    #[serde(rename = "current")]
    pub current: i32,
    /// The minimum number of crew members required to maintain the ship.
    #[serde(rename = "required")]
    pub required: i32,
    /// The maximum number of crew members the ship can support.
    #[serde(rename = "capacity")]
    pub capacity: i32,
    /// The rotation of crew shifts. A stricter shift improves the ship's performance. A more relaxed shift improves the crew's morale.
    #[serde(rename = "rotation")]
    pub rotation: Rotation,
    /// A rough measure of the crew's morale. A higher morale means the crew is happier and more productive. A lower morale means the ship is more prone to accidents.
    #[serde(rename = "morale")]
    pub morale: i32,
    /// The amount of credits per crew member paid per hour. Wages are paid when a ship docks at a civilized waypoint.
    #[serde(rename = "wages")]
    pub wages: i32,
}

impl ShipCrew {
    /// The ship's crew service and maintain the ship's systems and equipment.
    pub fn new(
        current: i32,
        required: i32,
        capacity: i32,
        rotation: Rotation,
        morale: i32,
        wages: i32,
    ) -> ShipCrew {
        ShipCrew {
            current,
            required,
            capacity,
            rotation,
            morale,
            wages,
        }
    }
}
/// The rotation of crew shifts. A stricter shift improves the ship's performance. A more relaxed shift improves the crew's morale.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Rotation {
    #[serde(rename = "STRICT")]
    Strict,
    #[serde(rename = "RELAXED")]
    Relaxed,
}

impl Default for Rotation {
    fn default() -> Rotation {
        Self::Strict
    }
}
