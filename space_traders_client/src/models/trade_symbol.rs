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
use strum_macros::EnumString;

/// TradeSymbol : The good's symbol.
/// The good's symbol.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    sqlx::Type,
    EnumString,
)]
#[sqlx(type_name = "trade_symbol")]
pub enum TradeSymbol {
    #[serde(rename = "PRECIOUS_STONES")]
    #[sqlx(rename = "PRECIOUS_STONES")]
    #[strum(serialize = "PRECIOUS_STONES")]
    PreciousStones,
    #[serde(rename = "QUARTZ_SAND")]
    #[sqlx(rename = "QUARTZ_SAND")]
    #[strum(serialize = "QUARTZ_SAND")]
    QuartzSand,
    #[serde(rename = "SILICON_CRYSTALS")]
    #[sqlx(rename = "SILICON_CRYSTALS")]
    #[strum(serialize = "SILICON_CRYSTALS")]
    SiliconCrystals,
    #[serde(rename = "AMMONIA_ICE")]
    #[sqlx(rename = "AMMONIA_ICE")]
    #[strum(serialize = "AMMONIA_ICE")]
    AmmoniaIce,
    #[serde(rename = "LIQUID_HYDROGEN")]
    #[sqlx(rename = "LIQUID_HYDROGEN")]
    #[strum(serialize = "LIQUID_HYDROGEN")]
    LiquidHydrogen,
    #[serde(rename = "LIQUID_NITROGEN")]
    #[sqlx(rename = "LIQUID_NITROGEN")]
    #[strum(serialize = "LIQUID_NITROGEN")]
    LiquidNitrogen,
    #[serde(rename = "ICE_WATER")]
    #[sqlx(rename = "ICE_WATER")]
    #[strum(serialize = "ICE_WATER")]
    IceWater,
    #[serde(rename = "EXOTIC_MATTER")]
    #[sqlx(rename = "EXOTIC_MATTER")]
    #[strum(serialize = "EXOTIC_MATTER")]
    ExoticMatter,
    #[serde(rename = "ADVANCED_CIRCUITRY")]
    #[sqlx(rename = "ADVANCED_CIRCUITRY")]
    #[strum(serialize = "ADVANCED_CIRCUITRY")]
    AdvancedCircuitry,
    #[serde(rename = "GRAVITON_EMITTERS")]
    #[sqlx(rename = "GRAVITON_EMITTERS")]
    #[strum(serialize = "GRAVITON_EMITTERS")]
    GravitonEmitters,
    #[serde(rename = "IRON")]
    #[sqlx(rename = "IRON")]
    #[strum(serialize = "IRON")]
    Iron,
    #[serde(rename = "IRON_ORE")]
    #[sqlx(rename = "IRON_ORE")]
    #[strum(serialize = "IRON_ORE")]
    IronOre,
    #[serde(rename = "COPPER")]
    #[sqlx(rename = "COPPER")]
    #[strum(serialize = "COPPER")]
    Copper,
    #[serde(rename = "COPPER_ORE")]
    #[sqlx(rename = "COPPER_ORE")]
    #[strum(serialize = "COPPER_ORE")]
    CopperOre,
    #[serde(rename = "ALUMINUM")]
    #[sqlx(rename = "ALUMINUM")]
    #[strum(serialize = "ALUMINUM")]
    Aluminum,
    #[serde(rename = "ALUMINUM_ORE")]
    #[sqlx(rename = "ALUMINUM_ORE")]
    #[strum(serialize = "ALUMINUM_ORE")]
    AluminumOre,
    #[serde(rename = "SILVER")]
    #[sqlx(rename = "SILVER")]
    #[strum(serialize = "SILVER")]
    Silver,
    #[serde(rename = "SILVER_ORE")]
    #[sqlx(rename = "SILVER_ORE")]
    #[strum(serialize = "SILVER_ORE")]
    SilverOre,
    #[serde(rename = "GOLD")]
    #[sqlx(rename = "GOLD")]
    #[strum(serialize = "GOLD")]
    Gold,
    #[serde(rename = "GOLD_ORE")]
    #[sqlx(rename = "GOLD_ORE")]
    #[strum(serialize = "GOLD_ORE")]
    GoldOre,
    #[serde(rename = "PLATINUM")]
    #[sqlx(rename = "PLATINUM")]
    #[strum(serialize = "PLATINUM")]
    Platinum,
    #[serde(rename = "PLATINUM_ORE")]
    #[sqlx(rename = "PLATINUM_ORE")]
    #[strum(serialize = "PLATINUM_ORE")]
    PlatinumOre,
    #[serde(rename = "DIAMONDS")]
    #[sqlx(rename = "DIAMONDS")]
    #[strum(serialize = "DIAMONDS")]
    Diamonds,
    #[serde(rename = "URANITE")]
    #[sqlx(rename = "URANITE")]
    #[strum(serialize = "URANITE")]
    Uranite,
    #[serde(rename = "URANITE_ORE")]
    #[sqlx(rename = "URANITE_ORE")]
    #[strum(serialize = "URANITE_ORE")]
    UraniteOre,
    #[serde(rename = "MERITIUM")]
    #[sqlx(rename = "MERITIUM")]
    #[strum(serialize = "MERITIUM")]
    Meritium,
    #[serde(rename = "MERITIUM_ORE")]
    #[sqlx(rename = "MERITIUM_ORE")]
    #[strum(serialize = "MERITIUM_ORE")]
    MeritiumOre,
    #[serde(rename = "HYDROCARBON")]
    #[sqlx(rename = "HYDROCARBON")]
    #[strum(serialize = "HYDROCARBON")]
    Hydrocarbon,
    #[serde(rename = "ANTIMATTER")]
    #[sqlx(rename = "ANTIMATTER")]
    #[strum(serialize = "ANTIMATTER")]
    Antimatter,
    #[serde(rename = "FAB_MATS")]
    #[sqlx(rename = "FAB_MATS")]
    #[strum(serialize = "FAB_MATS")]
    FabMats,
    #[serde(rename = "FERTILIZERS")]
    #[sqlx(rename = "FERTILIZERS")]
    #[strum(serialize = "FERTILIZERS")]
    Fertilizers,
    #[serde(rename = "FABRICS")]
    #[sqlx(rename = "FABRICS")]
    #[strum(serialize = "FABRICS")]
    Fabrics,
    #[serde(rename = "FOOD")]
    #[sqlx(rename = "FOOD")]
    #[strum(serialize = "FOOD")]
    Food,
    #[serde(rename = "JEWELRY")]
    #[sqlx(rename = "JEWELRY")]
    #[strum(serialize = "JEWELRY")]
    Jewelry,
    #[serde(rename = "MACHINERY")]
    #[sqlx(rename = "MACHINERY")]
    #[strum(serialize = "MACHINERY")]
    Machinery,
    #[serde(rename = "FIREARMS")]
    #[sqlx(rename = "FIREARMS")]
    #[strum(serialize = "FIREARMS")]
    Firearms,
    #[serde(rename = "ASSAULT_RIFLES")]
    #[sqlx(rename = "ASSAULT_RIFLES")]
    #[strum(serialize = "ASSAULT_RIFLES")]
    AssaultRifles,
    #[serde(rename = "MILITARY_EQUIPMENT")]
    #[sqlx(rename = "MILITARY_EQUIPMENT")]
    #[strum(serialize = "MILITARY_EQUIPMENT")]
    MilitaryEquipment,
    #[serde(rename = "EXPLOSIVES")]
    #[sqlx(rename = "EXPLOSIVES")]
    #[strum(serialize = "EXPLOSIVES")]
    Explosives,
    #[serde(rename = "LAB_INSTRUMENTS")]
    #[sqlx(rename = "LAB_INSTRUMENTS")]
    #[strum(serialize = "LAB_INSTRUMENTS")]
    LabInstruments,
    #[serde(rename = "AMMUNITION")]
    #[sqlx(rename = "AMMUNITION")]
    #[strum(serialize = "AMMUNITION")]
    Ammunition,
    #[serde(rename = "ELECTRONICS")]
    #[sqlx(rename = "ELECTRONICS")]
    #[strum(serialize = "ELECTRONICS")]
    Electronics,
    #[serde(rename = "SHIP_PLATING")]
    #[sqlx(rename = "SHIP_PLATING")]
    #[strum(serialize = "SHIP_PLATING")]
    ShipPlating,
    #[serde(rename = "SHIP_PARTS")]
    #[sqlx(rename = "SHIP_PARTS")]
    #[strum(serialize = "SHIP_PARTS")]
    ShipParts,
    #[serde(rename = "EQUIPMENT")]
    #[sqlx(rename = "EQUIPMENT")]
    #[strum(serialize = "EQUIPMENT")]
    Equipment,
    #[serde(rename = "FUEL")]
    #[sqlx(rename = "FUEL")]
    #[strum(serialize = "FUEL")]
    Fuel,
    #[serde(rename = "MEDICINE")]
    #[sqlx(rename = "MEDICINE")]
    #[strum(serialize = "MEDICINE")]
    Medicine,
    #[serde(rename = "DRUGS")]
    #[sqlx(rename = "DRUGS")]
    #[strum(serialize = "DRUGS")]
    Drugs,
    #[serde(rename = "CLOTHING")]
    #[sqlx(rename = "CLOTHING")]
    #[strum(serialize = "CLOTHING")]
    Clothing,
    #[serde(rename = "MICROPROCESSORS")]
    #[sqlx(rename = "MICROPROCESSORS")]
    #[strum(serialize = "MICROPROCESSORS")]
    Microprocessors,
    #[serde(rename = "PLASTICS")]
    #[sqlx(rename = "PLASTICS")]
    #[strum(serialize = "PLASTICS")]
    Plastics,
    #[serde(rename = "POLYNUCLEOTIDES")]
    #[sqlx(rename = "POLYNUCLEOTIDES")]
    #[strum(serialize = "POLYNUCLEOTIDES")]
    Polynucleotides,
    #[serde(rename = "BIOCOMPOSITES")]
    #[sqlx(rename = "BIOCOMPOSITES")]
    #[strum(serialize = "BIOCOMPOSITES")]
    Biocomposites,
    #[serde(rename = "QUANTUM_STABILIZERS")]
    #[sqlx(rename = "QUANTUM_STABILIZERS")]
    #[strum(serialize = "QUANTUM_STABILIZERS")]
    QuantumStabilizers,
    #[serde(rename = "NANOBOTS")]
    #[sqlx(rename = "NANOBOTS")]
    #[strum(serialize = "NANOBOTS")]
    Nanobots,
    #[serde(rename = "AI_MAINFRAMES")]
    #[sqlx(rename = "AI_MAINFRAMES")]
    #[strum(serialize = "AI_MAINFRAMES")]
    AiMainframes,
    #[serde(rename = "QUANTUM_DRIVES")]
    #[sqlx(rename = "QUANTUM_DRIVES")]
    #[strum(serialize = "QUANTUM_DRIVES")]
    QuantumDrives,
    #[serde(rename = "ROBOTIC_DRONES")]
    #[sqlx(rename = "ROBOTIC_DRONES")]
    #[strum(serialize = "ROBOTIC_DRONES")]
    RoboticDrones,
    #[serde(rename = "CYBER_IMPLANTS")]
    #[sqlx(rename = "CYBER_IMPLANTS")]
    #[strum(serialize = "CYBER_IMPLANTS")]
    CyberImplants,
    #[serde(rename = "GENE_THERAPEUTICS")]
    #[sqlx(rename = "GENE_THERAPEUTICS")]
    #[strum(serialize = "GENE_THERAPEUTICS")]
    GeneTherapeutics,
    #[serde(rename = "NEURAL_CHIPS")]
    #[sqlx(rename = "NEURAL_CHIPS")]
    #[strum(serialize = "NEURAL_CHIPS")]
    NeuralChips,
    #[serde(rename = "MOOD_REGULATORS")]
    #[sqlx(rename = "MOOD_REGULATORS")]
    #[strum(serialize = "MOOD_REGULATORS")]
    MoodRegulators,
    #[serde(rename = "VIRAL_AGENTS")]
    #[sqlx(rename = "VIRAL_AGENTS")]
    #[strum(serialize = "VIRAL_AGENTS")]
    ViralAgents,
    #[serde(rename = "MICRO_FUSION_GENERATORS")]
    #[sqlx(rename = "MICRO_FUSION_GENERATORS")]
    #[strum(serialize = "MICRO_FUSION_GENERATORS")]
    MicroFusionGenerators,
    #[serde(rename = "SUPERGRAINS")]
    #[sqlx(rename = "SUPERGRAINS")]
    #[strum(serialize = "SUPERGRAINS")]
    Supergrains,
    #[serde(rename = "LASER_RIFLES")]
    #[sqlx(rename = "LASER_RIFLES")]
    #[strum(serialize = "LASER_RIFLES")]
    LaserRifles,
    #[serde(rename = "HOLOGRAPHICS")]
    #[sqlx(rename = "HOLOGRAPHICS")]
    #[strum(serialize = "HOLOGRAPHICS")]
    Holographics,
    #[serde(rename = "SHIP_SALVAGE")]
    #[sqlx(rename = "SHIP_SALVAGE")]
    #[strum(serialize = "SHIP_SALVAGE")]
    ShipSalvage,
    #[serde(rename = "RELIC_TECH")]
    #[sqlx(rename = "RELIC_TECH")]
    #[strum(serialize = "RELIC_TECH")]
    RelicTech,
    #[serde(rename = "NOVEL_LIFEFORMS")]
    #[sqlx(rename = "NOVEL_LIFEFORMS")]
    #[strum(serialize = "NOVEL_LIFEFORMS")]
    NovelLifeforms,
    #[serde(rename = "BOTANICAL_SPECIMENS")]
    #[sqlx(rename = "BOTANICAL_SPECIMENS")]
    #[strum(serialize = "BOTANICAL_SPECIMENS")]
    BotanicalSpecimens,
    #[serde(rename = "CULTURAL_ARTIFACTS")]
    #[sqlx(rename = "CULTURAL_ARTIFACTS")]
    #[strum(serialize = "CULTURAL_ARTIFACTS")]
    CulturalArtifacts,
    #[serde(rename = "FRAME_PROBE")]
    #[sqlx(rename = "FRAME_PROBE")]
    #[strum(serialize = "FRAME_PROBE")]
    FrameProbe,
    #[serde(rename = "FRAME_DRONE")]
    #[sqlx(rename = "FRAME_DRONE")]
    #[strum(serialize = "FRAME_DRONE")]
    FrameDrone,
    #[serde(rename = "FRAME_INTERCEPTOR")]
    #[sqlx(rename = "FRAME_INTERCEPTOR")]
    #[strum(serialize = "FRAME_INTERCEPTOR")]
    FrameInterceptor,
    #[serde(rename = "FRAME_RACER")]
    #[sqlx(rename = "FRAME_RACER")]
    #[strum(serialize = "FRAME_RACER")]
    FrameRacer,
    #[serde(rename = "FRAME_FIGHTER")]
    #[sqlx(rename = "FRAME_FIGHTER")]
    #[strum(serialize = "FRAME_FIGHTER")]
    FrameFighter,
    #[serde(rename = "FRAME_FRIGATE")]
    #[sqlx(rename = "FRAME_FRIGATE")]
    #[strum(serialize = "FRAME_FRIGATE")]
    FrameFrigate,
    #[serde(rename = "FRAME_SHUTTLE")]
    #[sqlx(rename = "FRAME_SHUTTLE")]
    #[strum(serialize = "FRAME_SHUTTLE")]
    FrameShuttle,
    #[serde(rename = "FRAME_EXPLORER")]
    #[sqlx(rename = "FRAME_EXPLORER")]
    #[strum(serialize = "FRAME_EXPLORER")]
    FrameExplorer,
    #[serde(rename = "FRAME_MINER")]
    #[sqlx(rename = "FRAME_MINER")]
    #[strum(serialize = "FRAME_MINER")]
    FrameMiner,
    #[serde(rename = "FRAME_LIGHT_FREIGHTER")]
    #[sqlx(rename = "FRAME_LIGHT_FREIGHTER")]
    #[strum(serialize = "FRAME_LIGHT_FREIGHTER")]
    FrameLightFreighter,
    #[serde(rename = "FRAME_HEAVY_FREIGHTER")]
    #[sqlx(rename = "FRAME_HEAVY_FREIGHTER")]
    #[strum(serialize = "FRAME_HEAVY_FREIGHTER")]
    FrameHeavyFreighter,
    #[serde(rename = "FRAME_TRANSPORT")]
    #[sqlx(rename = "FRAME_TRANSPORT")]
    #[strum(serialize = "FRAME_TRANSPORT")]
    FrameTransport,
    #[serde(rename = "FRAME_DESTROYER")]
    #[sqlx(rename = "FRAME_DESTROYER")]
    #[strum(serialize = "FRAME_DESTROYER")]
    FrameDestroyer,
    #[serde(rename = "FRAME_CRUISER")]
    #[sqlx(rename = "FRAME_CRUISER")]
    #[strum(serialize = "FRAME_CRUISER")]
    FrameCruiser,
    #[serde(rename = "FRAME_CARRIER")]
    #[sqlx(rename = "FRAME_CARRIER")]
    #[strum(serialize = "FRAME_CARRIER")]
    FrameCarrier,
    #[serde(rename = "FRAME_BULK_FREIGHTER")]
    #[sqlx(rename = "FRAME_BULK_FREIGHTER")]
    #[strum(serialize = "FRAME_BULK_FREIGHTER")]
    FrameBulkFreighter,
    #[serde(rename = "REACTOR_SOLAR_I")]
    #[sqlx(rename = "REACTOR_SOLAR_I")]
    #[strum(serialize = "REACTOR_SOLAR_I")]
    ReactorSolarI,
    #[serde(rename = "REACTOR_FUSION_I")]
    #[sqlx(rename = "REACTOR_FUSION_I")]
    #[strum(serialize = "REACTOR_FUSION_I")]
    ReactorFusionI,
    #[serde(rename = "REACTOR_FISSION_I")]
    #[sqlx(rename = "REACTOR_FISSION_I")]
    #[strum(serialize = "REACTOR_FISSION_I")]
    ReactorFissionI,
    #[serde(rename = "REACTOR_CHEMICAL_I")]
    #[sqlx(rename = "REACTOR_CHEMICAL_I")]
    #[strum(serialize = "REACTOR_CHEMICAL_I")]
    ReactorChemicalI,
    #[serde(rename = "REACTOR_ANTIMATTER_I")]
    #[sqlx(rename = "REACTOR_ANTIMATTER_I")]
    #[strum(serialize = "REACTOR_ANTIMATTER_I")]
    ReactorAntimatterI,
    #[serde(rename = "ENGINE_IMPULSE_DRIVE_I")]
    #[sqlx(rename = "ENGINE_IMPULSE_DRIVE_I")]
    #[strum(serialize = "ENGINE_IMPULSE_DRIVE_I")]
    EngineImpulseDriveI,
    #[serde(rename = "ENGINE_ION_DRIVE_I")]
    #[sqlx(rename = "ENGINE_ION_DRIVE_I")]
    #[strum(serialize = "ENGINE_ION_DRIVE_I")]
    EngineIonDriveI,
    #[serde(rename = "ENGINE_ION_DRIVE_II")]
    #[sqlx(rename = "ENGINE_ION_DRIVE_II")]
    #[strum(serialize = "ENGINE_ION_DRIVE_II")]
    EngineIonDriveIi,
    #[serde(rename = "ENGINE_HYPER_DRIVE_I")]
    #[sqlx(rename = "ENGINE_HYPER_DRIVE_I")]
    #[strum(serialize = "ENGINE_HYPER_DRIVE_I")]
    EngineHyperDriveI,
    #[serde(rename = "MODULE_MINERAL_PROCESSOR_I")]
    #[sqlx(rename = "MODULE_MINERAL_PROCESSOR_I")]
    #[strum(serialize = "MODULE_MINERAL_PROCESSOR_I")]
    ModuleMineralProcessorI,
    #[serde(rename = "MODULE_GAS_PROCESSOR_I")]
    #[sqlx(rename = "MODULE_GAS_PROCESSOR_I")]
    #[strum(serialize = "MODULE_GAS_PROCESSOR_I")]
    ModuleGasProcessorI,
    #[serde(rename = "MODULE_CARGO_HOLD_I")]
    #[sqlx(rename = "MODULE_CARGO_HOLD_I")]
    #[strum(serialize = "MODULE_CARGO_HOLD_I")]
    ModuleCargoHoldI,
    #[serde(rename = "MODULE_CARGO_HOLD_II")]
    #[sqlx(rename = "MODULE_CARGO_HOLD_II")]
    #[strum(serialize = "MODULE_CARGO_HOLD_II")]
    ModuleCargoHoldIi,
    #[serde(rename = "MODULE_CARGO_HOLD_III")]
    #[sqlx(rename = "MODULE_CARGO_HOLD_III")]
    #[strum(serialize = "MODULE_CARGO_HOLD_III")]
    ModuleCargoHoldIii,
    #[serde(rename = "MODULE_CREW_QUARTERS_I")]
    #[sqlx(rename = "MODULE_CREW_QUARTERS_I")]
    #[strum(serialize = "MODULE_CREW_QUARTERS_I")]
    ModuleCrewQuartersI,
    #[serde(rename = "MODULE_ENVOY_QUARTERS_I")]
    #[sqlx(rename = "MODULE_ENVOY_QUARTERS_I")]
    #[strum(serialize = "MODULE_ENVOY_QUARTERS_I")]
    ModuleEnvoyQuartersI,
    #[serde(rename = "MODULE_PASSENGER_CABIN_I")]
    #[sqlx(rename = "MODULE_PASSENGER_CABIN_I")]
    #[strum(serialize = "MODULE_PASSENGER_CABIN_I")]
    ModulePassengerCabinI,
    #[serde(rename = "MODULE_MICRO_REFINERY_I")]
    #[sqlx(rename = "MODULE_MICRO_REFINERY_I")]
    #[strum(serialize = "MODULE_MICRO_REFINERY_I")]
    ModuleMicroRefineryI,
    #[serde(rename = "MODULE_SCIENCE_LAB_I")]
    #[sqlx(rename = "MODULE_SCIENCE_LAB_I")]
    #[strum(serialize = "MODULE_SCIENCE_LAB_I")]
    ModuleScienceLabI,
    #[serde(rename = "MODULE_JUMP_DRIVE_I")]
    #[sqlx(rename = "MODULE_JUMP_DRIVE_I")]
    #[strum(serialize = "MODULE_JUMP_DRIVE_I")]
    ModuleJumpDriveI,
    #[serde(rename = "MODULE_JUMP_DRIVE_II")]
    #[sqlx(rename = "MODULE_JUMP_DRIVE_II")]
    #[strum(serialize = "MODULE_JUMP_DRIVE_II")]
    ModuleJumpDriveIi,
    #[serde(rename = "MODULE_JUMP_DRIVE_III")]
    #[sqlx(rename = "MODULE_JUMP_DRIVE_III")]
    #[strum(serialize = "MODULE_JUMP_DRIVE_III")]
    ModuleJumpDriveIii,
    #[serde(rename = "MODULE_WARP_DRIVE_I")]
    #[sqlx(rename = "MODULE_WARP_DRIVE_I")]
    #[strum(serialize = "MODULE_WARP_DRIVE_I")]
    ModuleWarpDriveI,
    #[serde(rename = "MODULE_WARP_DRIVE_II")]
    #[sqlx(rename = "MODULE_WARP_DRIVE_II")]
    #[strum(serialize = "MODULE_WARP_DRIVE_II")]
    ModuleWarpDriveIi,
    #[serde(rename = "MODULE_WARP_DRIVE_III")]
    #[sqlx(rename = "MODULE_WARP_DRIVE_III")]
    #[strum(serialize = "MODULE_WARP_DRIVE_III")]
    ModuleWarpDriveIii,
    #[serde(rename = "MODULE_SHIELD_GENERATOR_I")]
    #[sqlx(rename = "MODULE_SHIELD_GENERATOR_I")]
    #[strum(serialize = "MODULE_SHIELD_GENERATOR_I")]
    ModuleShieldGeneratorI,
    #[serde(rename = "MODULE_SHIELD_GENERATOR_II")]
    #[sqlx(rename = "MODULE_SHIELD_GENERATOR_II")]
    #[strum(serialize = "MODULE_SHIELD_GENERATOR_II")]
    ModuleShieldGeneratorIi,
    #[serde(rename = "MODULE_ORE_REFINERY_I")]
    #[sqlx(rename = "MODULE_ORE_REFINERY_I")]
    #[strum(serialize = "MODULE_ORE_REFINERY_I")]
    ModuleOreRefineryI,
    #[serde(rename = "MODULE_FUEL_REFINERY_I")]
    #[sqlx(rename = "MODULE_FUEL_REFINERY_I")]
    #[strum(serialize = "MODULE_FUEL_REFINERY_I")]
    ModuleFuelRefineryI,
    #[serde(rename = "MOUNT_GAS_SIPHON_I")]
    #[sqlx(rename = "MOUNT_GAS_SIPHON_I")]
    #[strum(serialize = "MOUNT_GAS_SIPHON_I")]
    MountGasSiphonI,
    #[serde(rename = "MOUNT_GAS_SIPHON_II")]
    #[sqlx(rename = "MOUNT_GAS_SIPHON_II")]
    #[strum(serialize = "MOUNT_GAS_SIPHON_II")]
    MountGasSiphonIi,
    #[serde(rename = "MOUNT_GAS_SIPHON_III")]
    #[sqlx(rename = "MOUNT_GAS_SIPHON_III")]
    #[strum(serialize = "MOUNT_GAS_SIPHON_III")]
    MountGasSiphonIii,
    #[serde(rename = "MOUNT_SURVEYOR_I")]
    #[sqlx(rename = "MOUNT_SURVEYOR_I")]
    #[strum(serialize = "MOUNT_SURVEYOR_I")]
    MountSurveyorI,
    #[serde(rename = "MOUNT_SURVEYOR_II")]
    #[sqlx(rename = "MOUNT_SURVEYOR_II")]
    #[strum(serialize = "MOUNT_SURVEYOR_II")]
    MountSurveyorIi,
    #[serde(rename = "MOUNT_SURVEYOR_III")]
    #[sqlx(rename = "MOUNT_SURVEYOR_III")]
    #[strum(serialize = "MOUNT_SURVEYOR_III")]
    MountSurveyorIii,
    #[serde(rename = "MOUNT_SENSOR_ARRAY_I")]
    #[sqlx(rename = "MOUNT_SENSOR_ARRAY_I")]
    #[strum(serialize = "MOUNT_SENSOR_ARRAY_I")]
    MountSensorArrayI,
    #[serde(rename = "MOUNT_SENSOR_ARRAY_II")]
    #[sqlx(rename = "MOUNT_SENSOR_ARRAY_II")]
    #[strum(serialize = "MOUNT_SENSOR_ARRAY_II")]
    MountSensorArrayIi,
    #[serde(rename = "MOUNT_SENSOR_ARRAY_III")]
    #[sqlx(rename = "MOUNT_SENSOR_ARRAY_III")]
    #[strum(serialize = "MOUNT_SENSOR_ARRAY_III")]
    MountSensorArrayIii,
    #[serde(rename = "MOUNT_MINING_LASER_I")]
    #[sqlx(rename = "MOUNT_MINING_LASER_I")]
    #[strum(serialize = "MOUNT_MINING_LASER_I")]
    MountMiningLaserI,
    #[serde(rename = "MOUNT_MINING_LASER_II")]
    #[sqlx(rename = "MOUNT_MINING_LASER_II")]
    #[strum(serialize = "MOUNT_MINING_LASER_II")]
    MountMiningLaserIi,
    #[serde(rename = "MOUNT_MINING_LASER_III")]
    #[sqlx(rename = "MOUNT_MINING_LASER_III")]
    #[strum(serialize = "MOUNT_MINING_LASER_III")]
    MountMiningLaserIii,
    #[serde(rename = "MOUNT_LASER_CANNON_I")]
    #[sqlx(rename = "MOUNT_LASER_CANNON_I")]
    #[strum(serialize = "MOUNT_LASER_CANNON_I")]
    MountLaserCannonI,
    #[serde(rename = "MOUNT_MISSILE_LAUNCHER_I")]
    #[sqlx(rename = "MOUNT_MISSILE_LAUNCHER_I")]
    #[strum(serialize = "MOUNT_MISSILE_LAUNCHER_I")]
    MountMissileLauncherI,
    #[serde(rename = "MOUNT_TURRET_I")]
    #[sqlx(rename = "MOUNT_TURRET_I")]
    #[strum(serialize = "MOUNT_TURRET_I")]
    MountTurretI,
    #[serde(rename = "SHIP_PROBE")]
    #[sqlx(rename = "SHIP_PROBE")]
    #[strum(serialize = "SHIP_PROBE")]
    ShipProbe,
    #[serde(rename = "SHIP_MINING_DRONE")]
    #[sqlx(rename = "SHIP_MINING_DRONE")]
    #[strum(serialize = "SHIP_MINING_DRONE")]
    ShipMiningDrone,
    #[serde(rename = "SHIP_SIPHON_DRONE")]
    #[sqlx(rename = "SHIP_SIPHON_DRONE")]
    #[strum(serialize = "SHIP_SIPHON_DRONE")]
    ShipSiphonDrone,
    #[serde(rename = "SHIP_INTERCEPTOR")]
    #[sqlx(rename = "SHIP_INTERCEPTOR")]
    #[strum(serialize = "SHIP_INTERCEPTOR")]
    ShipInterceptor,
    #[serde(rename = "SHIP_LIGHT_HAULER")]
    #[sqlx(rename = "SHIP_LIGHT_HAULER")]
    #[strum(serialize = "SHIP_LIGHT_HAULER")]
    ShipLightHauler,
    #[serde(rename = "SHIP_COMMAND_FRIGATE")]
    #[sqlx(rename = "SHIP_COMMAND_FRIGATE")]
    #[strum(serialize = "SHIP_COMMAND_FRIGATE")]
    ShipCommandFrigate,
    #[serde(rename = "SHIP_EXPLORER")]
    #[sqlx(rename = "SHIP_EXPLORER")]
    #[strum(serialize = "SHIP_EXPLORER")]
    ShipExplorer,
    #[serde(rename = "SHIP_HEAVY_FREIGHTER")]
    #[sqlx(rename = "SHIP_HEAVY_FREIGHTER")]
    #[strum(serialize = "SHIP_HEAVY_FREIGHTER")]
    ShipHeavyFreighter,
    #[serde(rename = "SHIP_LIGHT_SHUTTLE")]
    #[sqlx(rename = "SHIP_LIGHT_SHUTTLE")]
    #[strum(serialize = "SHIP_LIGHT_SHUTTLE")]
    ShipLightShuttle,
    #[serde(rename = "SHIP_ORE_HOUND")]
    #[sqlx(rename = "SHIP_ORE_HOUND")]
    #[strum(serialize = "SHIP_ORE_HOUND")]
    ShipOreHound,
    #[serde(rename = "SHIP_REFINING_FREIGHTER")]
    #[sqlx(rename = "SHIP_REFINING_FREIGHTER")]
    #[strum(serialize = "SHIP_REFINING_FREIGHTER")]
    ShipRefiningFreighter,
    #[serde(rename = "SHIP_SURVEYOR")]
    #[sqlx(rename = "SHIP_SURVEYOR")]
    #[strum(serialize = "SHIP_SURVEYOR")]
    ShipSurveyor,
    #[serde(rename = "SHIP_BULK_FREIGHTER")]
    #[sqlx(rename = "SHIP_BULK_FREIGHTER")]
    #[strum(serialize = "SHIP_BULK_FREIGHTER")]
    ShipBulkFreighter,
}

impl From<models::ship_mount::Deposits> for TradeSymbol {
    fn from(value: models::ship_mount::Deposits) -> Self {
        match value {
            models::ship_mount::Deposits::QuartzSand => Self::QuartzSand,
            models::ship_mount::Deposits::SiliconCrystals => Self::SiliconCrystals,
            models::ship_mount::Deposits::PreciousStones => Self::PreciousStones,
            models::ship_mount::Deposits::IceWater => Self::IceWater,
            models::ship_mount::Deposits::AmmoniaIce => Self::AmmoniaIce,
            models::ship_mount::Deposits::IronOre => Self::IronOre,
            models::ship_mount::Deposits::CopperOre => Self::CopperOre,
            models::ship_mount::Deposits::SilverOre => Self::SilverOre,
            models::ship_mount::Deposits::AluminumOre => Self::AluminumOre,
            models::ship_mount::Deposits::GoldOre => Self::GoldOre,
            models::ship_mount::Deposits::PlatinumOre => Self::PlatinumOre,
            models::ship_mount::Deposits::Diamonds => Self::Diamonds,
            models::ship_mount::Deposits::UraniteOre => Self::UraniteOre,
            models::ship_mount::Deposits::MeritiumOre => Self::MeritiumOre,
        }
    }
}

impl std::fmt::Display for TradeSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::PreciousStones => write!(f, "PRECIOUS_STONES"),
            Self::QuartzSand => write!(f, "QUARTZ_SAND"),
            Self::SiliconCrystals => write!(f, "SILICON_CRYSTALS"),
            Self::AmmoniaIce => write!(f, "AMMONIA_ICE"),
            Self::LiquidHydrogen => write!(f, "LIQUID_HYDROGEN"),
            Self::LiquidNitrogen => write!(f, "LIQUID_NITROGEN"),
            Self::IceWater => write!(f, "ICE_WATER"),
            Self::ExoticMatter => write!(f, "EXOTIC_MATTER"),
            Self::AdvancedCircuitry => write!(f, "ADVANCED_CIRCUITRY"),
            Self::GravitonEmitters => write!(f, "GRAVITON_EMITTERS"),
            Self::Iron => write!(f, "IRON"),
            Self::IronOre => write!(f, "IRON_ORE"),
            Self::Copper => write!(f, "COPPER"),
            Self::CopperOre => write!(f, "COPPER_ORE"),
            Self::Aluminum => write!(f, "ALUMINUM"),
            Self::AluminumOre => write!(f, "ALUMINUM_ORE"),
            Self::Silver => write!(f, "SILVER"),
            Self::SilverOre => write!(f, "SILVER_ORE"),
            Self::Gold => write!(f, "GOLD"),
            Self::GoldOre => write!(f, "GOLD_ORE"),
            Self::Platinum => write!(f, "PLATINUM"),
            Self::PlatinumOre => write!(f, "PLATINUM_ORE"),
            Self::Diamonds => write!(f, "DIAMONDS"),
            Self::Uranite => write!(f, "URANITE"),
            Self::UraniteOre => write!(f, "URANITE_ORE"),
            Self::Meritium => write!(f, "MERITIUM"),
            Self::MeritiumOre => write!(f, "MERITIUM_ORE"),
            Self::Hydrocarbon => write!(f, "HYDROCARBON"),
            Self::Antimatter => write!(f, "ANTIMATTER"),
            Self::FabMats => write!(f, "FAB_MATS"),
            Self::Fertilizers => write!(f, "FERTILIZERS"),
            Self::Fabrics => write!(f, "FABRICS"),
            Self::Food => write!(f, "FOOD"),
            Self::Jewelry => write!(f, "JEWELRY"),
            Self::Machinery => write!(f, "MACHINERY"),
            Self::Firearms => write!(f, "FIREARMS"),
            Self::AssaultRifles => write!(f, "ASSAULT_RIFLES"),
            Self::MilitaryEquipment => write!(f, "MILITARY_EQUIPMENT"),
            Self::Explosives => write!(f, "EXPLOSIVES"),
            Self::LabInstruments => write!(f, "LAB_INSTRUMENTS"),
            Self::Ammunition => write!(f, "AMMUNITION"),
            Self::Electronics => write!(f, "ELECTRONICS"),
            Self::ShipPlating => write!(f, "SHIP_PLATING"),
            Self::ShipParts => write!(f, "SHIP_PARTS"),
            Self::Equipment => write!(f, "EQUIPMENT"),
            Self::Fuel => write!(f, "FUEL"),
            Self::Medicine => write!(f, "MEDICINE"),
            Self::Drugs => write!(f, "DRUGS"),
            Self::Clothing => write!(f, "CLOTHING"),
            Self::Microprocessors => write!(f, "MICROPROCESSORS"),
            Self::Plastics => write!(f, "PLASTICS"),
            Self::Polynucleotides => write!(f, "POLYNUCLEOTIDES"),
            Self::Biocomposites => write!(f, "BIOCOMPOSITES"),
            Self::QuantumStabilizers => write!(f, "QUANTUM_STABILIZERS"),
            Self::Nanobots => write!(f, "NANOBOTS"),
            Self::AiMainframes => write!(f, "AI_MAINFRAMES"),
            Self::QuantumDrives => write!(f, "QUANTUM_DRIVES"),
            Self::RoboticDrones => write!(f, "ROBOTIC_DRONES"),
            Self::CyberImplants => write!(f, "CYBER_IMPLANTS"),
            Self::GeneTherapeutics => write!(f, "GENE_THERAPEUTICS"),
            Self::NeuralChips => write!(f, "NEURAL_CHIPS"),
            Self::MoodRegulators => write!(f, "MOOD_REGULATORS"),
            Self::ViralAgents => write!(f, "VIRAL_AGENTS"),
            Self::MicroFusionGenerators => write!(f, "MICRO_FUSION_GENERATORS"),
            Self::Supergrains => write!(f, "SUPERGRAINS"),
            Self::LaserRifles => write!(f, "LASER_RIFLES"),
            Self::Holographics => write!(f, "HOLOGRAPHICS"),
            Self::ShipSalvage => write!(f, "SHIP_SALVAGE"),
            Self::RelicTech => write!(f, "RELIC_TECH"),
            Self::NovelLifeforms => write!(f, "NOVEL_LIFEFORMS"),
            Self::BotanicalSpecimens => write!(f, "BOTANICAL_SPECIMENS"),
            Self::CulturalArtifacts => write!(f, "CULTURAL_ARTIFACTS"),
            Self::FrameProbe => write!(f, "FRAME_PROBE"),
            Self::FrameDrone => write!(f, "FRAME_DRONE"),
            Self::FrameInterceptor => write!(f, "FRAME_INTERCEPTOR"),
            Self::FrameRacer => write!(f, "FRAME_RACER"),
            Self::FrameFighter => write!(f, "FRAME_FIGHTER"),
            Self::FrameFrigate => write!(f, "FRAME_FRIGATE"),
            Self::FrameShuttle => write!(f, "FRAME_SHUTTLE"),
            Self::FrameExplorer => write!(f, "FRAME_EXPLORER"),
            Self::FrameMiner => write!(f, "FRAME_MINER"),
            Self::FrameLightFreighter => write!(f, "FRAME_LIGHT_FREIGHTER"),
            Self::FrameHeavyFreighter => write!(f, "FRAME_HEAVY_FREIGHTER"),
            Self::FrameTransport => write!(f, "FRAME_TRANSPORT"),
            Self::FrameDestroyer => write!(f, "FRAME_DESTROYER"),
            Self::FrameCruiser => write!(f, "FRAME_CRUISER"),
            Self::FrameCarrier => write!(f, "FRAME_CARRIER"),
            Self::FrameBulkFreighter => write!(f, "FRAME_BULK_FREIGHTER"),
            Self::ReactorSolarI => write!(f, "REACTOR_SOLAR_I"),
            Self::ReactorFusionI => write!(f, "REACTOR_FUSION_I"),
            Self::ReactorFissionI => write!(f, "REACTOR_FISSION_I"),
            Self::ReactorChemicalI => write!(f, "REACTOR_CHEMICAL_I"),
            Self::ReactorAntimatterI => write!(f, "REACTOR_ANTIMATTER_I"),
            Self::EngineImpulseDriveI => write!(f, "ENGINE_IMPULSE_DRIVE_I"),
            Self::EngineIonDriveI => write!(f, "ENGINE_ION_DRIVE_I"),
            Self::EngineIonDriveIi => write!(f, "ENGINE_ION_DRIVE_II"),
            Self::EngineHyperDriveI => write!(f, "ENGINE_HYPER_DRIVE_I"),
            Self::ModuleMineralProcessorI => write!(f, "MODULE_MINERAL_PROCESSOR_I"),
            Self::ModuleGasProcessorI => write!(f, "MODULE_GAS_PROCESSOR_I"),
            Self::ModuleCargoHoldI => write!(f, "MODULE_CARGO_HOLD_I"),
            Self::ModuleCargoHoldIi => write!(f, "MODULE_CARGO_HOLD_II"),
            Self::ModuleCargoHoldIii => write!(f, "MODULE_CARGO_HOLD_III"),
            Self::ModuleCrewQuartersI => write!(f, "MODULE_CREW_QUARTERS_I"),
            Self::ModuleEnvoyQuartersI => write!(f, "MODULE_ENVOY_QUARTERS_I"),
            Self::ModulePassengerCabinI => write!(f, "MODULE_PASSENGER_CABIN_I"),
            Self::ModuleMicroRefineryI => write!(f, "MODULE_MICRO_REFINERY_I"),
            Self::ModuleScienceLabI => write!(f, "MODULE_SCIENCE_LAB_I"),
            Self::ModuleJumpDriveI => write!(f, "MODULE_JUMP_DRIVE_I"),
            Self::ModuleJumpDriveIi => write!(f, "MODULE_JUMP_DRIVE_II"),
            Self::ModuleJumpDriveIii => write!(f, "MODULE_JUMP_DRIVE_III"),
            Self::ModuleWarpDriveI => write!(f, "MODULE_WARP_DRIVE_I"),
            Self::ModuleWarpDriveIi => write!(f, "MODULE_WARP_DRIVE_II"),
            Self::ModuleWarpDriveIii => write!(f, "MODULE_WARP_DRIVE_III"),
            Self::ModuleShieldGeneratorI => write!(f, "MODULE_SHIELD_GENERATOR_I"),
            Self::ModuleShieldGeneratorIi => write!(f, "MODULE_SHIELD_GENERATOR_II"),
            Self::ModuleOreRefineryI => write!(f, "MODULE_ORE_REFINERY_I"),
            Self::ModuleFuelRefineryI => write!(f, "MODULE_FUEL_REFINERY_I"),
            Self::MountGasSiphonI => write!(f, "MOUNT_GAS_SIPHON_I"),
            Self::MountGasSiphonIi => write!(f, "MOUNT_GAS_SIPHON_II"),
            Self::MountGasSiphonIii => write!(f, "MOUNT_GAS_SIPHON_III"),
            Self::MountSurveyorI => write!(f, "MOUNT_SURVEYOR_I"),
            Self::MountSurveyorIi => write!(f, "MOUNT_SURVEYOR_II"),
            Self::MountSurveyorIii => write!(f, "MOUNT_SURVEYOR_III"),
            Self::MountSensorArrayI => write!(f, "MOUNT_SENSOR_ARRAY_I"),
            Self::MountSensorArrayIi => write!(f, "MOUNT_SENSOR_ARRAY_II"),
            Self::MountSensorArrayIii => write!(f, "MOUNT_SENSOR_ARRAY_III"),
            Self::MountMiningLaserI => write!(f, "MOUNT_MINING_LASER_I"),
            Self::MountMiningLaserIi => write!(f, "MOUNT_MINING_LASER_II"),
            Self::MountMiningLaserIii => write!(f, "MOUNT_MINING_LASER_III"),
            Self::MountLaserCannonI => write!(f, "MOUNT_LASER_CANNON_I"),
            Self::MountMissileLauncherI => write!(f, "MOUNT_MISSILE_LAUNCHER_I"),
            Self::MountTurretI => write!(f, "MOUNT_TURRET_I"),
            Self::ShipProbe => write!(f, "SHIP_PROBE"),
            Self::ShipMiningDrone => write!(f, "SHIP_MINING_DRONE"),
            Self::ShipSiphonDrone => write!(f, "SHIP_SIPHON_DRONE"),
            Self::ShipInterceptor => write!(f, "SHIP_INTERCEPTOR"),
            Self::ShipLightHauler => write!(f, "SHIP_LIGHT_HAULER"),
            Self::ShipCommandFrigate => write!(f, "SHIP_COMMAND_FRIGATE"),
            Self::ShipExplorer => write!(f, "SHIP_EXPLORER"),
            Self::ShipHeavyFreighter => write!(f, "SHIP_HEAVY_FREIGHTER"),
            Self::ShipLightShuttle => write!(f, "SHIP_LIGHT_SHUTTLE"),
            Self::ShipOreHound => write!(f, "SHIP_ORE_HOUND"),
            Self::ShipRefiningFreighter => write!(f, "SHIP_REFINING_FREIGHTER"),
            Self::ShipSurveyor => write!(f, "SHIP_SURVEYOR"),
            Self::ShipBulkFreighter => write!(f, "SHIP_BULK_FREIGHTER"),
        }
    }
}

impl Default for TradeSymbol {
    fn default() -> TradeSymbol {
        Self::PreciousStones
    }
}
