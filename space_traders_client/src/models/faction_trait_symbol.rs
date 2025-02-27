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

/// FactionTraitSymbol : The unique identifier of the trait.
/// The unique identifier of the trait.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum FactionTraitSymbol {
    #[serde(rename = "BUREAUCRATIC")]
    Bureaucratic,
    #[serde(rename = "SECRETIVE")]
    Secretive,
    #[serde(rename = "CAPITALISTIC")]
    Capitalistic,
    #[serde(rename = "INDUSTRIOUS")]
    Industrious,
    #[serde(rename = "PEACEFUL")]
    Peaceful,
    #[serde(rename = "DISTRUSTFUL")]
    Distrustful,
    #[serde(rename = "WELCOMING")]
    Welcoming,
    #[serde(rename = "SMUGGLERS")]
    Smugglers,
    #[serde(rename = "SCAVENGERS")]
    Scavengers,
    #[serde(rename = "REBELLIOUS")]
    Rebellious,
    #[serde(rename = "EXILES")]
    Exiles,
    #[serde(rename = "PIRATES")]
    Pirates,
    #[serde(rename = "RAIDERS")]
    Raiders,
    #[serde(rename = "CLAN")]
    Clan,
    #[serde(rename = "GUILD")]
    Guild,
    #[serde(rename = "DOMINION")]
    Dominion,
    #[serde(rename = "FRINGE")]
    Fringe,
    #[serde(rename = "FORSAKEN")]
    Forsaken,
    #[serde(rename = "ISOLATED")]
    Isolated,
    #[serde(rename = "LOCALIZED")]
    Localized,
    #[serde(rename = "ESTABLISHED")]
    Established,
    #[serde(rename = "NOTABLE")]
    Notable,
    #[serde(rename = "DOMINANT")]
    Dominant,
    #[serde(rename = "INESCAPABLE")]
    Inescapable,
    #[serde(rename = "INNOVATIVE")]
    Innovative,
    #[serde(rename = "BOLD")]
    Bold,
    #[serde(rename = "VISIONARY")]
    Visionary,
    #[serde(rename = "CURIOUS")]
    Curious,
    #[serde(rename = "DARING")]
    Daring,
    #[serde(rename = "EXPLORATORY")]
    Exploratory,
    #[serde(rename = "RESOURCEFUL")]
    Resourceful,
    #[serde(rename = "FLEXIBLE")]
    Flexible,
    #[serde(rename = "COOPERATIVE")]
    Cooperative,
    #[serde(rename = "UNITED")]
    United,
    #[serde(rename = "STRATEGIC")]
    Strategic,
    #[serde(rename = "INTELLIGENT")]
    Intelligent,
    #[serde(rename = "RESEARCH_FOCUSED")]
    ResearchFocused,
    #[serde(rename = "COLLABORATIVE")]
    Collaborative,
    #[serde(rename = "PROGRESSIVE")]
    Progressive,
    #[serde(rename = "MILITARISTIC")]
    Militaristic,
    #[serde(rename = "TECHNOLOGICALLY_ADVANCED")]
    TechnologicallyAdvanced,
    #[serde(rename = "AGGRESSIVE")]
    Aggressive,
    #[serde(rename = "IMPERIALISTIC")]
    Imperialistic,
    #[serde(rename = "TREASURE_HUNTERS")]
    TreasureHunters,
    #[serde(rename = "DEXTEROUS")]
    Dexterous,
    #[serde(rename = "UNPREDICTABLE")]
    Unpredictable,
    #[serde(rename = "BRUTAL")]
    Brutal,
    #[serde(rename = "FLEETING")]
    Fleeting,
    #[serde(rename = "ADAPTABLE")]
    Adaptable,
    #[serde(rename = "SELF_SUFFICIENT")]
    SelfSufficient,
    #[serde(rename = "DEFENSIVE")]
    Defensive,
    #[serde(rename = "PROUD")]
    Proud,
    #[serde(rename = "DIVERSE")]
    Diverse,
    #[serde(rename = "INDEPENDENT")]
    Independent,
    #[serde(rename = "SELF_INTERESTED")]
    SelfInterested,
    #[serde(rename = "FRAGMENTED")]
    Fragmented,
    #[serde(rename = "COMMERCIAL")]
    Commercial,
    #[serde(rename = "FREE_MARKETS")]
    FreeMarkets,
    #[serde(rename = "ENTREPRENEURIAL")]
    Entrepreneurial,
}

impl std::fmt::Display for FactionTraitSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Bureaucratic => write!(f, "BUREAUCRATIC"),
            Self::Secretive => write!(f, "SECRETIVE"),
            Self::Capitalistic => write!(f, "CAPITALISTIC"),
            Self::Industrious => write!(f, "INDUSTRIOUS"),
            Self::Peaceful => write!(f, "PEACEFUL"),
            Self::Distrustful => write!(f, "DISTRUSTFUL"),
            Self::Welcoming => write!(f, "WELCOMING"),
            Self::Smugglers => write!(f, "SMUGGLERS"),
            Self::Scavengers => write!(f, "SCAVENGERS"),
            Self::Rebellious => write!(f, "REBELLIOUS"),
            Self::Exiles => write!(f, "EXILES"),
            Self::Pirates => write!(f, "PIRATES"),
            Self::Raiders => write!(f, "RAIDERS"),
            Self::Clan => write!(f, "CLAN"),
            Self::Guild => write!(f, "GUILD"),
            Self::Dominion => write!(f, "DOMINION"),
            Self::Fringe => write!(f, "FRINGE"),
            Self::Forsaken => write!(f, "FORSAKEN"),
            Self::Isolated => write!(f, "ISOLATED"),
            Self::Localized => write!(f, "LOCALIZED"),
            Self::Established => write!(f, "ESTABLISHED"),
            Self::Notable => write!(f, "NOTABLE"),
            Self::Dominant => write!(f, "DOMINANT"),
            Self::Inescapable => write!(f, "INESCAPABLE"),
            Self::Innovative => write!(f, "INNOVATIVE"),
            Self::Bold => write!(f, "BOLD"),
            Self::Visionary => write!(f, "VISIONARY"),
            Self::Curious => write!(f, "CURIOUS"),
            Self::Daring => write!(f, "DARING"),
            Self::Exploratory => write!(f, "EXPLORATORY"),
            Self::Resourceful => write!(f, "RESOURCEFUL"),
            Self::Flexible => write!(f, "FLEXIBLE"),
            Self::Cooperative => write!(f, "COOPERATIVE"),
            Self::United => write!(f, "UNITED"),
            Self::Strategic => write!(f, "STRATEGIC"),
            Self::Intelligent => write!(f, "INTELLIGENT"),
            Self::ResearchFocused => write!(f, "RESEARCH_FOCUSED"),
            Self::Collaborative => write!(f, "COLLABORATIVE"),
            Self::Progressive => write!(f, "PROGRESSIVE"),
            Self::Militaristic => write!(f, "MILITARISTIC"),
            Self::TechnologicallyAdvanced => write!(f, "TECHNOLOGICALLY_ADVANCED"),
            Self::Aggressive => write!(f, "AGGRESSIVE"),
            Self::Imperialistic => write!(f, "IMPERIALISTIC"),
            Self::TreasureHunters => write!(f, "TREASURE_HUNTERS"),
            Self::Dexterous => write!(f, "DEXTEROUS"),
            Self::Unpredictable => write!(f, "UNPREDICTABLE"),
            Self::Brutal => write!(f, "BRUTAL"),
            Self::Fleeting => write!(f, "FLEETING"),
            Self::Adaptable => write!(f, "ADAPTABLE"),
            Self::SelfSufficient => write!(f, "SELF_SUFFICIENT"),
            Self::Defensive => write!(f, "DEFENSIVE"),
            Self::Proud => write!(f, "PROUD"),
            Self::Diverse => write!(f, "DIVERSE"),
            Self::Independent => write!(f, "INDEPENDENT"),
            Self::SelfInterested => write!(f, "SELF_INTERESTED"),
            Self::Fragmented => write!(f, "FRAGMENTED"),
            Self::Commercial => write!(f, "COMMERCIAL"),
            Self::FreeMarkets => write!(f, "FREE_MARKETS"),
            Self::Entrepreneurial => write!(f, "ENTREPRENEURIAL"),
        }
    }
}

impl Default for FactionTraitSymbol {
    fn default() -> FactionTraitSymbol {
        Self::Bureaucratic
    }
}
