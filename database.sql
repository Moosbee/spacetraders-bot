-- Database: spaceTrader
-- DROP DATABASE IF EXISTS "spaceTrader";
CREATE DATABASE "spaceTrader"
WITH
  OWNER = "spTrader" ENCODING = 'UTF8' LC_COLLATE = 'en_US.utf8' LC_CTYPE = 'en_US.utf8' TABLESPACE = pg_default CONNECTION
LIMIT
  = -1 IS_TEMPLATE = False;

USE "spaceTrader";

CREATE TYPE trade_symbol AS ENUM (
  'PreciousStones',
  'QuartzSand',
  'SiliconCrystals',
  'AmmoniaIce',
  'LiquidHydrogen',
  'LiquidNitrogen',
  'IceWater',
  'ExoticMatter',
  'AdvancedCircuitry',
  'GravitonEmitters',
  'Iron',
  'IronOre',
  'Copper',
  'CopperOre',
  'Aluminum',
  'AluminumOre',
  'Silver',
  'SilverOre',
  'Gold',
  'GoldOre',
  'Platinum',
  'PlatinumOre',
  'Diamonds',
  'Uranite',
  'UraniteOre',
  'Meritium',
  'MeritiumOre',
  'Hydrocarbon',
  'Antimatter',
  'FabMats',
  'Fertilizers',
  'Fabrics',
  'Food',
  'Jewelry',
  'Machinery',
  'Firearms',
  'AssaultRifles',
  'MilitaryEquipment',
  'Explosives',
  'LabInstruments',
  'Ammunition',
  'Electronics',
  'ShipPlating',
  'ShipParts',
  'Equipment',
  'Fuel',
  'Medicine',
  'Drugs',
  'Clothing',
  'Microprocessors',
  'Plastics',
  'Polynucleotides',
  'Biocomposites',
  'QuantumStabilizers',
  'Nanobots',
  'AiMainframes',
  'QuantumDrives',
  'RoboticDrones',
  'CyberImplants',
  'GeneTherapeutics',
  'NeuralChips',
  'MoodRegulators',
  'ViralAgents',
  'MicroFusionGenerators',
  'Supergrains',
  'LaserRifles',
  'Holographics',
  'ShipSalvage',
  'RelicTech',
  'NovelLifeforms',
  'BotanicalSpecimens',
  'CulturalArtifacts',
  'FrameProbe',
  'FrameDrone',
  'FrameInterceptor',
  'FrameRacer',
  'FrameFighter',
  'FrameFrigate',
  'FrameShuttle',
  'FrameExplorer',
  'FrameMiner',
  'FrameLightFreighter',
  'FrameHeavyFreighter',
  'FrameTransport',
  'FrameDestroyer',
  'FrameCruiser',
  'FrameCarrier',
  'ReactorSolarI',
  'ReactorFusionI',
  'ReactorFissionI',
  'ReactorChemicalI',
  'ReactorAntimatterI',
  'EngineImpulseDriveI',
  'EngineIonDriveI',
  'EngineIonDriveIi',
  'EngineHyperDriveI',
  'ModuleMineralProcessorI',
  'ModuleGasProcessorI',
  'ModuleCargoHoldI',
  'ModuleCargoHoldIi',
  'ModuleCargoHoldIii',
  'ModuleCrewQuartersI',
  'ModuleEnvoyQuartersI',
  'ModulePassengerCabinI',
  'ModuleMicroRefineryI',
  'ModuleScienceLabI',
  'ModuleJumpDriveI',
  'ModuleJumpDriveIi',
  'ModuleJumpDriveIii',
  'ModuleWarpDriveI',
  'ModuleWarpDriveIi',
  'ModuleWarpDriveIii',
  'ModuleShieldGeneratorI',
  'ModuleShieldGeneratorIi',
  'ModuleOreRefineryI',
  'ModuleFuelRefineryI',
  'MountGasSiphonI',
  'MountGasSiphonIi',
  'MountGasSiphonIii',
  'MountSurveyorI',
  'MountSurveyorIi',
  'MountSurveyorIii',
  'MountSensorArrayI',
  'MountSensorArrayIi',
  'MountSensorArrayIii',
  'MountMiningLaserI',
  'MountMiningLaserIi',
  'MountMiningLaserIii',
  'MountLaserCannonI',
  'MountMissileLauncherI',
  'MountTurretI',
  'ShipProbe',
  'ShipMiningDrone',
  'ShipSiphonDrone',
  'ShipInterceptor',
  'ShipLightHauler',
  'ShipCommandFrigate',
  'ShipExplorer',
  'ShipHeavyFreighter',
  'ShipLightShuttle',
  'ShipOreHound',
  'ShipRefiningFreighter',
  'ShipSurveyor'
);

CREATE TYPE market_transaction_type AS ENUM ('Purchase', 'Sell');

CREATE TYPE market_trade_good_type AS ENUM ('Export', 'Import', 'Exchange');

CREATE TYPE supply_level AS ENUM (
  'Scarce',
  'Limited',
  'Moderate',
  'High',
  'Abundant'
);

CREATE TYPE activity_level AS ENUM ('Weak', 'Growing', 'Strong', 'Restricted');

-- Table: public.waypoint
-- DROP TABLE IF EXISTS public.waypoint;
CREATE TABLE
  IF NOT EXISTS public.waypoint (
    symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
    system_symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
    created_at timestamp without time zone NOT NULL DEFAULT now (),
    CONSTRAINT waypoint_pkey PRIMARY KEY (symbol)
  );

-- Table: public.market_trade_good
-- DROP TABLE IF EXISTS public.market_trade_good;
CREATE TABLE
  IF NOT EXISTS public.market_trade_good (
    created_at timestamp without time zone NOT NULL DEFAULT now (),
    created timestamp without time zone NOT NULL DEFAULT now (),
    waypoint_symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
    symbol trade_symbol NOT NULL,
    type market_trade_good_type NOT NULL,
    trade_volume integer NOT NULL,
    supply supply_level NOT NULL,
    activity activity_level NOT NULL,
    purchase_price integer NOT NULL,
    sell_price integer NOT NULL,
    CONSTRAINT market_trade_good_pkey PRIMARY KEY (created, symbol, waypoint_symbol),
    CONSTRAINT market_trade_good_relation_1 FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
  );

-- Table: public.market_transaction
-- DROP TABLE IF EXISTS public.market_transaction;
CREATE TABLE
  IF NOT EXISTS public.market_transaction (
    waypoint_symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
    ship_symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
    type market_transaction_type NOT NULL,
    units integer NOT NULL,
    price_per_unit integer NOT NULL,
    total_price integer NOT NULL,
    "timestamp" character varying(255) COLLATE pg_catalog."default" NOT NULL,
    CONSTRAINT market_transaction_pkey PRIMARY KEY (waypoint_symbol, ship_symbol, "timestamp"),
    CONSTRAINT market_transaction_relation_1 FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
  );