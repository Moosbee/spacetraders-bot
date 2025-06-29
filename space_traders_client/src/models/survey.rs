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

/// Survey : A resource survey of a waypoint, detailing a specific extraction location and the types of resources that can be found there.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Survey {
    /// A unique signature for the location of this survey. This signature is verified when attempting an extraction using this survey.
    #[serde(rename = "signature")]
    pub signature: String,
    /// The symbol of the waypoint that this survey is for.
    #[serde(rename = "symbol")]
    pub symbol: String,
    /// A list of deposits that can be found at this location. A ship will extract one of these deposits when using this survey in an extraction request. If multiple deposits of the same type are present, the chance of extracting that deposit is increased.
    #[serde(rename = "deposits")]
    pub deposits: Vec<models::SurveyDeposit>,
    /// The date and time when the survey expires. After this date and time, the survey will no longer be available for extraction.
    #[serde(rename = "expiration")]
    pub expiration: String,
    #[serde(rename = "size")]
    pub size: models::SurveySize,
}

impl Survey {
    /// A resource survey of a waypoint, detailing a specific extraction location and the types of resources that can be found there.
    pub fn new(
        signature: String,
        symbol: String,
        deposits: Vec<models::SurveyDeposit>,
        expiration: String,
        size: models::SurveySize,
    ) -> Survey {
        Survey {
            signature,
            symbol,
            deposits,
            expiration,
            size,
        }
    }
}
