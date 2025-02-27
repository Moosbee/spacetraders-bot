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

/// ShipNavRoute : The routing information for the ship's most recent transit or current location.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipNavRoute {
    #[serde(rename = "destination")]
    pub destination: Box<models::ShipNavRouteWaypoint>,
    #[serde(rename = "origin")]
    pub origin: Box<models::ShipNavRouteWaypoint>,
    /// The date time of the ship's departure.
    #[serde(rename = "departureTime")]
    pub departure_time: String,
    /// The date time of the ship's arrival. If the ship is in-transit, this is the expected time of arrival.
    #[serde(rename = "arrival")]
    pub arrival: String,
}

impl ShipNavRoute {
    /// The routing information for the ship's most recent transit or current location.
    pub fn new(
        destination: models::ShipNavRouteWaypoint,
        origin: models::ShipNavRouteWaypoint,
        departure_time: String,
        arrival: String,
    ) -> ShipNavRoute {
        ShipNavRoute {
            destination: Box::new(destination),
            origin: Box::new(origin),
            departure_time,
            arrival,
        }
    }
}
