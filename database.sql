-- Database: spaceTrader
-- DROP DATABASE IF EXISTS "spaceTrader";
CREATE DATABASE "spaceTrader" WITH OWNER = "spTrader" ENCODING = 'UTF8' LC_COLLATE = 'en_US.utf8' LC_CTYPE = 'en_US.utf8' TABLESPACE = pg_default CONNECTION
LIMIT
  = -1 IS_TEMPLATE = False;

USE "spaceTrader";

CREATE TYPE trade_symbol AS ENUM (
  'PRECIOUS_STONES',
  'QUARTZ_SAND',
  'SILICON_CRYSTALS',
  'AMMONIA_ICE',
  'LIQUID_HYDROGEN',
  'LIQUID_NITROGEN',
  'ICE_WATER',
  'EXOTIC_MATTER',
  'ADVANCED_CIRCUITRY',
  'GRAVITON_EMITTERS',
  'IRON',
  'IRON_ORE',
  'COPPER',
  'COPPER_ORE',
  'ALUMINUM',
  'ALUMINUM_ORE',
  'SILVER',
  'SILVER_ORE',
  'GOLD',
  'GOLD_ORE',
  'PLATINUM',
  'PLATINUM_ORE',
  'DIAMONDS',
  'URANITE',
  'URANITE_ORE',
  'MERITIUM',
  'MERITIUM_ORE',
  'HYDROCARBON',
  'ANTIMATTER',
  'FAB_MATS',
  'FERTILIZERS',
  'FABRICS',
  'FOOD',
  'JEWELRY',
  'MACHINERY',
  'FIREARMS',
  'ASSAULT_RIFLES',
  'MILITARY_EQUIPMENT',
  'EXPLOSIVES',
  'LAB_INSTRUMENTS',
  'AMMUNITION',
  'ELECTRONICS',
  'SHIP_PLATING',
  'SHIP_PARTS',
  'EQUIPMENT',
  'FUEL',
  'MEDICINE',
  'DRUGS',
  'CLOTHING',
  'MICROPROCESSORS',
  'PLASTICS',
  'POLYNUCLEOTIDES',
  'BIOCOMPOSITES',
  'QUANTUM_STABILIZERS',
  'NANOBOTS',
  'AI_MAINFRAMES',
  'QUANTUM_DRIVES',
  'ROBOTIC_DRONES',
  'CYBER_IMPLANTS',
  'GENE_THERAPEUTICS',
  'NEURAL_CHIPS',
  'MOOD_REGULATORS',
  'VIRAL_AGENTS',
  'MICRO_FUSION_GENERATORS',
  'SUPERGRAINS',
  'LASER_RIFLES',
  'HOLOGRAPHICS',
  'SHIP_SALVAGE',
  'RELIC_TECH',
  'NOVEL_LIFEFORMS',
  'BOTANICAL_SPECIMENS',
  'CULTURAL_ARTIFACTS',
  'FRAME_PROBE',
  'FRAME_DRONE',
  'FRAME_INTERCEPTOR',
  'FRAME_RACER',
  'FRAME_FIGHTER',
  'FRAME_FRIGATE',
  'FRAME_SHUTTLE',
  'FRAME_EXPLORER',
  'FRAME_MINER',
  'FRAME_LIGHT_FREIGHTER',
  'FRAME_HEAVY_FREIGHTER',
  'FRAME_TRANSPORT',
  'FRAME_DESTROYER',
  'FRAME_CRUISER',
  'FRAME_CARRIER',
  'REACTOR_SOLAR_I',
  'REACTOR_FUSION_I',
  'REACTOR_FISSION_I',
  'REACTOR_CHEMICAL_I',
  'REACTOR_ANTIMATTER_I',
  'ENGINE_IMPULSE_DRIVE_I',
  'ENGINE_ION_DRIVE_I',
  'ENGINE_ION_DRIVE_II',
  'ENGINE_HYPER_DRIVE_I',
  'MODULE_MINERAL_PROCESSOR_I',
  'MODULE_GAS_PROCESSOR_I',
  'MODULE_CARGO_HOLD_I',
  'MODULE_CARGO_HOLD_II',
  'MODULE_CARGO_HOLD_III',
  'MODULE_CREW_QUARTERS_I',
  'MODULE_ENVOY_QUARTERS_I',
  'MODULE_PASSENGER_CABIN_I',
  'MODULE_MICRO_REFINERY_I',
  'MODULE_SCIENCE_LAB_I',
  'MODULE_JUMP_DRIVE_I',
  'MODULE_JUMP_DRIVE_II',
  'MODULE_JUMP_DRIVE_III',
  'MODULE_WARP_DRIVE_I',
  'MODULE_WARP_DRIVE_II',
  'MODULE_WARP_DRIVE_III',
  'MODULE_SHIELD_GENERATOR_I',
  'MODULE_SHIELD_GENERATOR_II',
  'MODULE_ORE_REFINERY_I',
  'MODULE_FUEL_REFINERY_I',
  'MOUNT_GAS_SIPHON_I',
  'MOUNT_GAS_SIPHON_II',
  'MOUNT_GAS_SIPHON_III',
  'MOUNT_SURVEYOR_I',
  'MOUNT_SURVEYOR_II',
  'MOUNT_SURVEYOR_III',
  'MOUNT_SENSOR_ARRAY_I',
  'MOUNT_SENSOR_ARRAY_II',
  'MOUNT_SENSOR_ARRAY_III',
  'MOUNT_MINING_LASER_I',
  'MOUNT_MINING_LASER_II',
  'MOUNT_MINING_LASER_III',
  'MOUNT_LASER_CANNON_I',
  'MOUNT_MISSILE_LAUNCHER_I',
  'MOUNT_TURRET_I',
  'SHIP_PROBE',
  'SHIP_MINING_DRONE',
  'SHIP_SIPHON_DRONE',
  'SHIP_INTERCEPTOR',
  'SHIP_LIGHT_HAULER',
  'SHIP_COMMAND_FRIGATE',
  'SHIP_EXPLORER',
  'SHIP_HEAVY_FREIGHTER',
  'SHIP_LIGHT_SHUTTLE',
  'SHIP_ORE_HOUND',
  'SHIP_REFINING_FREIGHTER',
  'SHIP_SURVEYOR'
);

CREATE TYPE market_transaction_type AS ENUM ('PURCHASE', 'SELL');

CREATE TYPE market_trade_good_type AS ENUM ('EXPORT', 'IMPORT', 'EXCHANGE');

CREATE TYPE supply_level AS ENUM (
  'SCARCE',
  'LIMITED',
  'MODERATE',
  'HIGH',
  'ABUNDANT'
);

CREATE TYPE activity_level AS ENUM ('WEAK', 'GROWING', 'STRONG', 'RESTRICTED');

-- Table: public.waypoint
-- DROP TABLE IF EXISTS public.waypoint;
CREATE TABLE IF NOT EXISTS public.waypoint (
  symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
  system_symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
  created_at timestamp without time zone NOT NULL DEFAULT now (),
  CONSTRAINT waypoint_pkey PRIMARY KEY (symbol)
);

-- Table: public.market_trade_good
-- DROP TABLE IF EXISTS public.market_trade_good;
CREATE TABLE IF NOT EXISTS public.market_trade_good (
  created_at timestamp without time zone NOT NULL DEFAULT now (),
  created timestamp without time zone NOT NULL DEFAULT now (),
  waypoint_symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
  symbol trade_symbol NOT NULL,
  type market_trade_good_type NOT NULL,
  trade_volume integer NOT NULL,
  supply supply_level NOT NULL,
  activity activity_level,
  purchase_price integer NOT NULL,
  sell_price integer NOT NULL,
  CONSTRAINT market_trade_good_pkey PRIMARY KEY (created, symbol, waypoint_symbol),
  CONSTRAINT market_trade_good_relation_1 FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
);

-- Table: public.market_transaction
-- DROP TABLE IF EXISTS public.market_transaction;
CREATE TABLE IF NOT EXISTS public.market_transaction (
  waypoint_symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
  ship_symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
  type market_transaction_type NOT NULL,
  units integer NOT NULL,
  price_per_unit integer NOT NULL,
  total_price integer NOT NULL,
  "timestamp" character varying(255) COLLATE pg_catalog."default" NOT NULL,
  trade_symbol trade_symbol NOT NULL,
  CONSTRAINT market_transaction_pkey PRIMARY KEY (
    waypoint_symbol,
    ship_symbol,
    trade_symbol,
    "timestamp"
  ),
  CONSTRAINT market_transaction_relation_1 FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
) TABLESPACE pg_default;

CREATE TABLE public.market_trade (
  waypoint_symbol character varying(255) NOT NULL,
  symbol trade_symbol NOT NULL,
  type market_trade_good_type NOT NULL,
  created_at timestamp without time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (created_at, symbol, waypoint_symbol)
);