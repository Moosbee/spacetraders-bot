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

/// ActivityLevel : The activity level of a trade good. If the good is an import, this represents how strong consumption is. If the good is an export, this represents how strong the production is for the good. When activity is strong, consumption or production is near maximum capacity. When activity is weak, consumption or production is near minimum capacity.
/// The activity level of a trade good. If the good is an import, this represents how strong consumption is. If the good is an export, this represents how strong the production is for the good. When activity is strong, consumption or production is near maximum capacity. When activity is weak, consumption or production is near minimum capacity.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "activity_level")]
pub enum ActivityLevel {
    #[serde(rename = "WEAK")]
    #[sqlx(rename = "WEAK")]
    Weak,
    #[serde(rename = "GROWING")]
    #[sqlx(rename = "GROWING")]
    Growing,
    #[serde(rename = "STRONG")]
    #[sqlx(rename = "STRONG")]
    Strong,
    #[serde(rename = "RESTRICTED")]
    #[sqlx(rename = "RESTRICTED")]
    Restricted,
}

impl std::fmt::Display for ActivityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Weak => write!(f, "WEAK"),
            Self::Growing => write!(f, "GROWING"),
            Self::Strong => write!(f, "STRONG"),
            Self::Restricted => write!(f, "RESTRICTED"),
        }
    }
}

impl Default for ActivityLevel {
    fn default() -> ActivityLevel {
        Self::Weak
    }
}
