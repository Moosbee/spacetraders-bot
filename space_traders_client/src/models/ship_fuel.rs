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

/// ShipFuel : Details of the ship's fuel tanks including how much fuel was consumed during the last transit or action.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipFuel {
    /// The current amount of fuel in the ship's tanks.
    #[serde(rename = "current")]
    pub current: i32,
    /// The maximum amount of fuel the ship's tanks can hold.
    #[serde(rename = "capacity")]
    pub capacity: i32,
    #[serde(rename = "consumed", skip_serializing_if = "Option::is_none")]
    pub consumed: Option<Box<models::ShipFuelConsumed>>,
}

impl ShipFuel {
    /// Details of the ship's fuel tanks including how much fuel was consumed during the last transit or action.
    pub fn new(current: i32, capacity: i32) -> ShipFuel {
        ShipFuel {
            current,
            capacity,
            consumed: None,
        }
    }
}
