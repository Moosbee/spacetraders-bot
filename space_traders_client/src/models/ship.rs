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

/// Ship : Ship details.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ship {
    /// The globally unique identifier of the ship in the following format: `[AGENT_SYMBOL]-[HEX_ID]`
    #[serde(rename = "symbol")]
    pub symbol: String,
    #[serde(rename = "registration")]
    pub registration: Box<models::ShipRegistration>,
    #[serde(rename = "nav")]
    pub nav: Box<models::ShipNav>,
    #[serde(rename = "crew")]
    pub crew: Box<models::ShipCrew>,
    #[serde(rename = "frame")]
    pub frame: Box<models::ShipFrame>,
    #[serde(rename = "reactor")]
    pub reactor: Box<models::ShipReactor>,
    #[serde(rename = "engine")]
    pub engine: Box<models::ShipEngine>,
    /// Modules installed in this ship.
    #[serde(rename = "modules")]
    pub modules: Vec<models::ShipModule>,
    /// Mounts installed in this ship.
    #[serde(rename = "mounts")]
    pub mounts: Vec<models::ShipMount>,
    #[serde(rename = "cargo")]
    pub cargo: Box<models::ShipCargo>,
    #[serde(rename = "fuel")]
    pub fuel: Box<models::ShipFuel>,
    #[serde(rename = "cooldown")]
    pub cooldown: Box<models::Cooldown>,
}

impl Ship {
    /// Ship details.
    pub fn new(
        symbol: String,
        registration: models::ShipRegistration,
        nav: models::ShipNav,
        crew: models::ShipCrew,
        frame: models::ShipFrame,
        reactor: models::ShipReactor,
        engine: models::ShipEngine,
        modules: Vec<models::ShipModule>,
        mounts: Vec<models::ShipMount>,
        cargo: models::ShipCargo,
        fuel: models::ShipFuel,
        cooldown: models::Cooldown,
    ) -> Ship {
        Ship {
            symbol,
            registration: Box::new(registration),
            nav: Box::new(nav),
            crew: Box::new(crew),
            frame: Box::new(frame),
            reactor: Box::new(reactor),
            engine: Box::new(engine),
            modules,
            mounts,
            cargo: Box::new(cargo),
            fuel: Box::new(fuel),
            cooldown: Box::new(cooldown),
        }
    }
}
