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

/// WaypointTraitSymbol : The unique identifier of the trait.
/// The unique identifier of the trait.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "waypoint_trait_symbol")]
pub enum WaypointTraitSymbol {
    #[serde(rename = "UNCHARTED")]
    #[sqlx(rename = "UNCHARTED")]
    Uncharted,
    #[serde(rename = "UNDER_CONSTRUCTION")]
    #[sqlx(rename = "UNDER_CONSTRUCTION")]
    UnderConstruction,
    #[serde(rename = "MARKETPLACE")]
    #[sqlx(rename = "MARKETPLACE")]
    Marketplace,
    #[serde(rename = "SHIPYARD")]
    #[sqlx(rename = "SHIPYARD")]
    Shipyard,
    #[serde(rename = "OUTPOST")]
    #[sqlx(rename = "OUTPOST")]
    Outpost,
    #[serde(rename = "SCATTERED_SETTLEMENTS")]
    #[sqlx(rename = "SCATTERED_SETTLEMENTS")]
    ScatteredSettlements,
    #[serde(rename = "SPRAWLING_CITIES")]
    #[sqlx(rename = "SPRAWLING_CITIES")]
    SprawlingCities,
    #[serde(rename = "MEGA_STRUCTURES")]
    #[sqlx(rename = "MEGA_STRUCTURES")]
    MegaStructures,
    #[serde(rename = "PIRATE_BASE")]
    #[sqlx(rename = "PIRATE_BASE")]
    PirateBase,
    #[serde(rename = "OVERCROWDED")]
    #[sqlx(rename = "OVERCROWDED")]
    Overcrowded,
    #[serde(rename = "HIGH_TECH")]
    #[sqlx(rename = "HIGH_TECH")]
    HighTech,
    #[serde(rename = "CORRUPT")]
    #[sqlx(rename = "CORRUPT")]
    Corrupt,
    #[serde(rename = "BUREAUCRATIC")]
    #[sqlx(rename = "BUREAUCRATIC")]
    Bureaucratic,
    #[serde(rename = "TRADING_HUB")]
    #[sqlx(rename = "TRADING_HUB")]
    TradingHub,
    #[serde(rename = "INDUSTRIAL")]
    #[sqlx(rename = "INDUSTRIAL")]
    Industrial,
    #[serde(rename = "BLACK_MARKET")]
    #[sqlx(rename = "BLACK_MARKET")]
    BlackMarket,
    #[serde(rename = "RESEARCH_FACILITY")]
    #[sqlx(rename = "RESEARCH_FACILITY")]
    ResearchFacility,
    #[serde(rename = "MILITARY_BASE")]
    #[sqlx(rename = "MILITARY_BASE")]
    MilitaryBase,
    #[serde(rename = "SURVEILLANCE_OUTPOST")]
    #[sqlx(rename = "SURVEILLANCE_OUTPOST")]
    SurveillanceOutpost,
    #[serde(rename = "EXPLORATION_OUTPOST")]
    #[sqlx(rename = "EXPLORATION_OUTPOST")]
    ExplorationOutpost,
    #[serde(rename = "MINERAL_DEPOSITS")]
    #[sqlx(rename = "MINERAL_DEPOSITS")]
    MineralDeposits,
    #[serde(rename = "COMMON_METAL_DEPOSITS")]
    #[sqlx(rename = "COMMON_METAL_DEPOSITS")]
    CommonMetalDeposits,
    #[serde(rename = "PRECIOUS_METAL_DEPOSITS")]
    #[sqlx(rename = "PRECIOUS_METAL_DEPOSITS")]
    PreciousMetalDeposits,
    #[serde(rename = "RARE_METAL_DEPOSITS")]
    #[sqlx(rename = "RARE_METAL_DEPOSITS")]
    RareMetalDeposits,
    #[serde(rename = "METHANE_POOLS")]
    #[sqlx(rename = "METHANE_POOLS")]
    MethanePools,
    #[serde(rename = "ICE_CRYSTALS")]
    #[sqlx(rename = "ICE_CRYSTALS")]
    IceCrystals,
    #[serde(rename = "EXPLOSIVE_GASES")]
    #[sqlx(rename = "EXPLOSIVE_GASES")]
    ExplosiveGases,
    #[serde(rename = "STRONG_MAGNETOSPHERE")]
    #[sqlx(rename = "STRONG_MAGNETOSPHERE")]
    StrongMagnetosphere,
    #[serde(rename = "VIBRANT_AURORAS")]
    #[sqlx(rename = "VIBRANT_AURORAS")]
    VibrantAuroras,
    #[serde(rename = "SALT_FLATS")]
    #[sqlx(rename = "SALT_FLATS")]
    SaltFlats,
    #[serde(rename = "CANYONS")]
    #[sqlx(rename = "CANYONS")]
    Canyons,
    #[serde(rename = "PERPETUAL_DAYLIGHT")]
    #[sqlx(rename = "PERPETUAL_DAYLIGHT")]
    PerpetualDaylight,
    #[serde(rename = "PERPETUAL_OVERCAST")]
    #[sqlx(rename = "PERPETUAL_OVERCAST")]
    PerpetualOvercast,
    #[serde(rename = "DRY_SEABEDS")]
    #[sqlx(rename = "DRY_SEABEDS")]
    DrySeabeds,
    #[serde(rename = "MAGMA_SEAS")]
    #[sqlx(rename = "MAGMA_SEAS")]
    MagmaSeas,
    #[serde(rename = "SUPERVOLCANOES")]
    #[sqlx(rename = "SUPERVOLCANOES")]
    Supervolcanoes,
    #[serde(rename = "ASH_CLOUDS")]
    #[sqlx(rename = "ASH_CLOUDS")]
    AshClouds,
    #[serde(rename = "VAST_RUINS")]
    #[sqlx(rename = "VAST_RUINS")]
    VastRuins,
    #[serde(rename = "MUTATED_FLORA")]
    #[sqlx(rename = "MUTATED_FLORA")]
    MutatedFlora,
    #[serde(rename = "TERRAFORMED")]
    #[sqlx(rename = "TERRAFORMED")]
    Terraformed,
    #[serde(rename = "EXTREME_TEMPERATURES")]
    #[sqlx(rename = "EXTREME_TEMPERATURES")]
    ExtremeTemperatures,
    #[serde(rename = "EXTREME_PRESSURE")]
    #[sqlx(rename = "EXTREME_PRESSURE")]
    ExtremePressure,
    #[serde(rename = "DIVERSE_LIFE")]
    #[sqlx(rename = "DIVERSE_LIFE")]
    DiverseLife,
    #[serde(rename = "SCARCE_LIFE")]
    #[sqlx(rename = "SCARCE_LIFE")]
    ScarceLife,
    #[serde(rename = "FOSSILS")]
    #[sqlx(rename = "FOSSILS")]
    Fossils,
    #[serde(rename = "WEAK_GRAVITY")]
    #[sqlx(rename = "WEAK_GRAVITY")]
    WeakGravity,
    #[serde(rename = "STRONG_GRAVITY")]
    #[sqlx(rename = "STRONG_GRAVITY")]
    StrongGravity,
    #[serde(rename = "CRUSHING_GRAVITY")]
    #[sqlx(rename = "CRUSHING_GRAVITY")]
    CrushingGravity,
    #[serde(rename = "TOXIC_ATMOSPHERE")]
    #[sqlx(rename = "TOXIC_ATMOSPHERE")]
    ToxicAtmosphere,
    #[serde(rename = "CORROSIVE_ATMOSPHERE")]
    #[sqlx(rename = "CORROSIVE_ATMOSPHERE")]
    CorrosiveAtmosphere,
    #[serde(rename = "BREATHABLE_ATMOSPHERE")]
    #[sqlx(rename = "BREATHABLE_ATMOSPHERE")]
    BreathableAtmosphere,
    #[serde(rename = "THIN_ATMOSPHERE")]
    #[sqlx(rename = "THIN_ATMOSPHERE")]
    ThinAtmosphere,
    #[serde(rename = "JOVIAN")]
    #[sqlx(rename = "JOVIAN")]
    Jovian,
    #[serde(rename = "ROCKY")]
    #[sqlx(rename = "ROCKY")]
    Rocky,
    #[serde(rename = "VOLCANIC")]
    #[sqlx(rename = "VOLCANIC")]
    Volcanic,
    #[serde(rename = "FROZEN")]
    #[sqlx(rename = "FROZEN")]
    Frozen,
    #[serde(rename = "SWAMP")]
    #[sqlx(rename = "SWAMP")]
    Swamp,
    #[serde(rename = "BARREN")]
    #[sqlx(rename = "BARREN")]
    Barren,
    #[serde(rename = "TEMPERATE")]
    #[sqlx(rename = "TEMPERATE")]
    Temperate,
    #[serde(rename = "JUNGLE")]
    #[sqlx(rename = "JUNGLE")]
    Jungle,
    #[serde(rename = "OCEAN")]
    #[sqlx(rename = "OCEAN")]
    Ocean,
    #[serde(rename = "RADIOACTIVE")]
    #[sqlx(rename = "RADIOACTIVE")]
    Radioactive,
    #[serde(rename = "MICRO_GRAVITY_ANOMALIES")]
    #[sqlx(rename = "MICRO_GRAVITY_ANOMALIES")]
    MicroGravityAnomalies,
    #[serde(rename = "DEBRIS_CLUSTER")]
    #[sqlx(rename = "DEBRIS_CLUSTER")]
    DebrisCluster,
    #[serde(rename = "DEEP_CRATERS")]
    #[sqlx(rename = "DEEP_CRATERS")]
    DeepCraters,
    #[serde(rename = "SHALLOW_CRATERS")]
    #[sqlx(rename = "SHALLOW_CRATERS")]
    ShallowCraters,
    #[serde(rename = "UNSTABLE_COMPOSITION")]
    #[sqlx(rename = "UNSTABLE_COMPOSITION")]
    UnstableComposition,
    #[serde(rename = "HOLLOWED_INTERIOR")]
    #[sqlx(rename = "HOLLOWED_INTERIOR")]
    HollowedInterior,
    #[serde(rename = "STRIPPED")]
    #[sqlx(rename = "STRIPPED")]
    Stripped,
}

impl std::fmt::Display for WaypointTraitSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Uncharted => write!(f, "UNCHARTED"),
            Self::UnderConstruction => write!(f, "UNDER_CONSTRUCTION"),
            Self::Marketplace => write!(f, "MARKETPLACE"),
            Self::Shipyard => write!(f, "SHIPYARD"),
            Self::Outpost => write!(f, "OUTPOST"),
            Self::ScatteredSettlements => write!(f, "SCATTERED_SETTLEMENTS"),
            Self::SprawlingCities => write!(f, "SPRAWLING_CITIES"),
            Self::MegaStructures => write!(f, "MEGA_STRUCTURES"),
            Self::PirateBase => write!(f, "PIRATE_BASE"),
            Self::Overcrowded => write!(f, "OVERCROWDED"),
            Self::HighTech => write!(f, "HIGH_TECH"),
            Self::Corrupt => write!(f, "CORRUPT"),
            Self::Bureaucratic => write!(f, "BUREAUCRATIC"),
            Self::TradingHub => write!(f, "TRADING_HUB"),
            Self::Industrial => write!(f, "INDUSTRIAL"),
            Self::BlackMarket => write!(f, "BLACK_MARKET"),
            Self::ResearchFacility => write!(f, "RESEARCH_FACILITY"),
            Self::MilitaryBase => write!(f, "MILITARY_BASE"),
            Self::SurveillanceOutpost => write!(f, "SURVEILLANCE_OUTPOST"),
            Self::ExplorationOutpost => write!(f, "EXPLORATION_OUTPOST"),
            Self::MineralDeposits => write!(f, "MINERAL_DEPOSITS"),
            Self::CommonMetalDeposits => write!(f, "COMMON_METAL_DEPOSITS"),
            Self::PreciousMetalDeposits => write!(f, "PRECIOUS_METAL_DEPOSITS"),
            Self::RareMetalDeposits => write!(f, "RARE_METAL_DEPOSITS"),
            Self::MethanePools => write!(f, "METHANE_POOLS"),
            Self::IceCrystals => write!(f, "ICE_CRYSTALS"),
            Self::ExplosiveGases => write!(f, "EXPLOSIVE_GASES"),
            Self::StrongMagnetosphere => write!(f, "STRONG_MAGNETOSPHERE"),
            Self::VibrantAuroras => write!(f, "VIBRANT_AURORAS"),
            Self::SaltFlats => write!(f, "SALT_FLATS"),
            Self::Canyons => write!(f, "CANYONS"),
            Self::PerpetualDaylight => write!(f, "PERPETUAL_DAYLIGHT"),
            Self::PerpetualOvercast => write!(f, "PERPETUAL_OVERCAST"),
            Self::DrySeabeds => write!(f, "DRY_SEABEDS"),
            Self::MagmaSeas => write!(f, "MAGMA_SEAS"),
            Self::Supervolcanoes => write!(f, "SUPERVOLCANOES"),
            Self::AshClouds => write!(f, "ASH_CLOUDS"),
            Self::VastRuins => write!(f, "VAST_RUINS"),
            Self::MutatedFlora => write!(f, "MUTATED_FLORA"),
            Self::Terraformed => write!(f, "TERRAFORMED"),
            Self::ExtremeTemperatures => write!(f, "EXTREME_TEMPERATURES"),
            Self::ExtremePressure => write!(f, "EXTREME_PRESSURE"),
            Self::DiverseLife => write!(f, "DIVERSE_LIFE"),
            Self::ScarceLife => write!(f, "SCARCE_LIFE"),
            Self::Fossils => write!(f, "FOSSILS"),
            Self::WeakGravity => write!(f, "WEAK_GRAVITY"),
            Self::StrongGravity => write!(f, "STRONG_GRAVITY"),
            Self::CrushingGravity => write!(f, "CRUSHING_GRAVITY"),
            Self::ToxicAtmosphere => write!(f, "TOXIC_ATMOSPHERE"),
            Self::CorrosiveAtmosphere => write!(f, "CORROSIVE_ATMOSPHERE"),
            Self::BreathableAtmosphere => write!(f, "BREATHABLE_ATMOSPHERE"),
            Self::ThinAtmosphere => write!(f, "THIN_ATMOSPHERE"),
            Self::Jovian => write!(f, "JOVIAN"),
            Self::Rocky => write!(f, "ROCKY"),
            Self::Volcanic => write!(f, "VOLCANIC"),
            Self::Frozen => write!(f, "FROZEN"),
            Self::Swamp => write!(f, "SWAMP"),
            Self::Barren => write!(f, "BARREN"),
            Self::Temperate => write!(f, "TEMPERATE"),
            Self::Jungle => write!(f, "JUNGLE"),
            Self::Ocean => write!(f, "OCEAN"),
            Self::Radioactive => write!(f, "RADIOACTIVE"),
            Self::MicroGravityAnomalies => write!(f, "MICRO_GRAVITY_ANOMALIES"),
            Self::DebrisCluster => write!(f, "DEBRIS_CLUSTER"),
            Self::DeepCraters => write!(f, "DEEP_CRATERS"),
            Self::ShallowCraters => write!(f, "SHALLOW_CRATERS"),
            Self::UnstableComposition => write!(f, "UNSTABLE_COMPOSITION"),
            Self::HollowedInterior => write!(f, "HOLLOWED_INTERIOR"),
            Self::Stripped => write!(f, "STRIPPED"),
        }
    }
}

impl Default for WaypointTraitSymbol {
    fn default() -> WaypointTraitSymbol {
        Self::Uncharted
    }
}
