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

/// Chart : The chart of a system or waypoint, which makes the location visible to other agents.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct Chart {
    /// The symbol of the waypoint.
    #[serde(rename = "waypointSymbol", skip_serializing_if = "Option::is_none")]
    pub waypoint_symbol: Option<String>,
    /// The agent that submitted the chart for this waypoint.
    #[serde(rename = "submittedBy", skip_serializing_if = "Option::is_none")]
    pub submitted_by: Option<String>,
    /// The time the chart for this waypoint was submitted.
    #[serde(rename = "submittedOn", skip_serializing_if = "Option::is_none")]
    pub submitted_on: Option<String>,
}

impl Chart {
    /// The chart of a system or waypoint, which makes the location visible to other agents.
    pub fn new() -> Chart {
        Chart {
            waypoint_symbol: None,
            submitted_by: None,
            submitted_on: None,
        }
    }
}
