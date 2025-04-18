-- Database: spaceTrader
-- DROP DATABASE IF EXISTS "spaceTrader";
CREATE DATABASE "spaceTrader" WITH OWNER = "spTrader" ENCODING = 'UTF8' LC_COLLATE = 'en_US.utf8' LC_CTYPE = 'en_US.utf8' TABLESPACE = pg_default CONNECTION
LIMIT = -1 IS_TEMPLATE = False;
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
CREATE TYPE contract_type AS ENUM ('PROCUREMENT', 'TRANSPORT', 'SHUTTLE');
CREATE TYPE ship_info_role AS ENUM (
  'Construction',
  'Trader',
  'Contract',
  'Scraper',
  'Mining',
  'Manuel'
);
CREATE TABLE public.ship_info (
  symbol character varying NOT NULL,
  display_name character varying NOT NULL,
  role ship_info_role NOT NULL,
  active boolean NOT NULL,
  PRIMARY KEY (symbol)
);
-- Table: public.agent
-- DROP TABLE IF EXISTS public.agent;
CREATE TABLE IF NOT EXISTS public.agent (
  id SERIAL PRIMARY KEY,
  symbol character varying COLLATE pg_catalog."default" NOT NULL,
  account_id character varying COLLATE pg_catalog."default",
  headquarters character varying COLLATE pg_catalog."default" NOT NULL,
  credits bigint NOT NULL,
  starting_faction character varying COLLATE pg_catalog."default" NOT NULL,
  ship_count integer NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now ()
) TABLESPACE pg_default;
-- Table: public.waypoint
-- DROP TABLE IF EXISTS public.waypoint;
CREATE TABLE IF NOT EXISTS public.waypoint (
  symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
  system_symbol character varying(255) COLLATE pg_catalog."default" NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now (),
  CONSTRAINT waypoint_pkey PRIMARY KEY (symbol)
);
-- Table: public.market_trade_good
-- DROP TABLE IF EXISTS public.market_trade_good;
CREATE TABLE IF NOT EXISTS public.market_trade_good (
  created_at timestamp with time zone NOT NULL DEFAULT now (),
  created timestamp with time zone NOT NULL DEFAULT now (),
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
CREATE TABLE public.market_trade (
  waypoint_symbol character varying(255) NOT NULL,
  symbol trade_symbol NOT NULL,
  type market_trade_good_type NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now (),
  PRIMARY KEY (created_at, symbol, waypoint_symbol),
  CONSTRAINT market_trade_relation_1 FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Table for the main contract details
CREATE TABLE contract (
  id character varying(255) PRIMARY KEY,
  faction_symbol character varying(255) NOT NULL,
  contract_type contract_type NOT NULL,
  accepted BOOLEAN NOT NULL DEFAULT false,
  fulfilled BOOLEAN NOT NULL DEFAULT false,
  deadline_to_accept character varying(255),
  on_accepted INTEGER NOT NULL,
  on_fulfilled INTEGER NOT NULL,
  deadline character varying(255) NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now ()
);
-- Table for contract delivery requirements
CREATE TABLE contract_delivery (
  contract_id character varying(255) NOT NULL,
  trade_symbol trade_symbol NOT NULL,
  destination_symbol character varying(255) NOT NULL,
  units_required INTEGER NOT NULL,
  units_fulfilled INTEGER NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now (),
  PRIMARY KEY (contract_id, trade_symbol, destination_symbol),
  CONSTRAINT contract_delivery_relation_1 FOREIGN KEY (contract_id) REFERENCES public.contract (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Table: public.trade_route
-- DROP TABLE IF EXISTS public.trade_route;
CREATE TABLE IF NOT EXISTS public.trade_route (
  id SERIAL PRIMARY KEY,
  symbol trade_symbol NOT NULL,
  ship_symbol character varying COLLATE pg_catalog."default" NOT NULL,
  purchase_waypoint character varying COLLATE pg_catalog."default" NOT NULL,
  sell_waypoint character varying COLLATE pg_catalog."default" NOT NULL,
  finished boolean NOT NULL DEFAULT false,
  trade_volume integer NOT NULL DEFAULT 1,
  predicted_purchase_price integer NOT NULL,
  predicted_sell_price integer NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now ()
) TABLESPACE pg_default;
-- Table: public.route
-- DROP TABLE IF EXISTS public.route;
CREATE TABLE IF NOT EXISTS public.route (
  id SERIAL PRIMARY KEY,
  "from" character varying COLLATE pg_catalog."default" NOT NULL,
  "to" character varying COLLATE pg_catalog."default" NOT NULL,
  distance double precision NOT NULL,
  nav_mode character varying COLLATE pg_catalog."default" NOT NULL,
  speed integer NOT NULL,
  fuel_cost integer NOT NULL,
  travel_time double precision NOT NULL,
  engine_condition double precision NOT NULL DEFAULT 1,
  frame_condition double precision NOT NULL,
  reactor_condition double precision NOT NULL,
  current_cargo integer NOT NULL,
  total_cargohold integer NOT NULL,
  CONSTRAINT route_relation_1 FOREIGN KEY ("from") REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT route_relation_2 FOREIGN KEY ("to") REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
) TABLESPACE pg_default;
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
  contract character varying(255) COLLATE pg_catalog."default",
  trade_route integer,
  mining character varying(255) COLLATE pg_catalog."default",
  CONSTRAINT market_transaction_pkey PRIMARY KEY (
    waypoint_symbol,
    ship_symbol,
    trade_symbol,
    "timestamp"
  ),
  CONSTRAINT market_transaction_relation_1 FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT market_transaction_relation_2 FOREIGN KEY (contract) REFERENCES public.contract (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT market_transaction_relation_3 FOREIGN KEY (trade_route) REFERENCES public.trade_route (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT market_transaction_check CHECK (
    contract IS NOT NULL
    AND trade_route IS NULL
    OR contract IS NULL
    AND trade_route IS NOT NULL
    OR contract IS NULL
    AND trade_route IS NULL
  ) NOT VALID
) TABLESPACE pg_default;
-- Select statements
-- Get all contracts and how much profit they made
SELECT contract.id,
  contract.faction_symbol,
  contract.contract_type,
  contract.accepted,
  contract.fulfilled,
  contract.deadline_to_accept,
  contract.on_accepted,
  contract.on_fulfilled,
  contract.deadline,
  contract.on_accepted + contract.on_fulfilled as "totalprofit",
  sum(market_transaction.total_price) as "total_expenses",
  contract.on_accepted + contract.on_fulfilled - sum(market_transaction.total_price) as "net_profit"
FROM public.contract
  join public.market_transaction ON market_transaction.contract = contract.id
group by contract.id
order by contract.deadline_to_accept ASC;
SELECT id,
  symbol,
  trade_route.ship_symbol,
  purchase_waypoint,
  sell_waypoint,
  finished,
  predicted_purchase_price,
  predicted_sell_price,
  created_at,
  sum(market_transaction.total_price) as "sum",
  sum(
    CASE
      WHEN market_transaction.type = 'PURCHASE' THEN market_transaction.total_price
      ELSE 0
    END
  ) as "expenses",
  sum(
    CASE
      WHEN market_transaction.type = 'PURCHASE' THEN 0
      ELSE market_transaction.total_price
    END
  ) as "income",
  sum(
    CASE
      WHEN market_transaction.type = 'PURCHASE' THEN (market_transaction.total_price * -1)
      ELSE market_transaction.total_price
    END
  ) as "profit"
FROM public.trade_route
  join public.market_transaction ON market_transaction.trade_route = trade_route.id
group by id
ORDER BY id ASC;