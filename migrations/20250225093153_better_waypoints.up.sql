-- Add up migration script here
-- symbol
-- done
-- system_symbol
-- done
-- x
-- y
-- r#type
CREATE TYPE waypoint_type AS ENUM (
  'PLANET',
  'GAS_GIANT',
  'MOON',
  'ORBITAL_STATION',
  'JUMP_GATE',
  'ASTEROID_FIELD',
  'ASTEROID',
  'ENGINEERED_ASTEROID',
  'ASTEROID_BASE',
  'NEBULA',
  'DEBRIS_FIELD',
  'GRAVITY_WELL',
  'ARTIFICIAL_GRAVITY_WELL',
  'FUEL_STATION'
);
-- traits
CREATE TYPE waypoint_trait_symbol AS ENUM (
  'UNCHARTED',
  'UNDER_CONSTRUCTION',
  'MARKETPLACE',
  'SHIPYARD',
  'OUTPOST',
  'SCATTERED_SETTLEMENTS',
  'SPRAWLING_CITIES',
  'MEGA_STRUCTURES',
  'PIRATE_BASE',
  'OVERCROWDED',
  'HIGH_TECH',
  'CORRUPT',
  'BUREAUCRATIC',
  'TRADING_HUB',
  'INDUSTRIAL',
  'BLACK_MARKET',
  'RESEARCH_FACILITY',
  'MILITARY_BASE',
  'SURVEILLANCE_OUTPOST',
  'EXPLORATION_OUTPOST',
  'MINERAL_DEPOSITS',
  'COMMON_METAL_DEPOSITS',
  'PRECIOUS_METAL_DEPOSITS',
  'RARE_METAL_DEPOSITS',
  'METHANE_POOLS',
  'ICE_CRYSTALS',
  'EXPLOSIVE_GASES',
  'STRONG_MAGNETOSPHERE',
  'VIBRANT_AURORAS',
  'SALT_FLATS',
  'CANYONS',
  'PERPETUAL_DAYLIGHT',
  'PERPETUAL_OVERCAST',
  'DRY_SEABEDS',
  'MAGMA_SEAS',
  'SUPERVOLCANOES',
  'ASH_CLOUDS',
  'VAST_RUINS',
  'MUTATED_FLORA',
  'TERRAFORMED',
  'EXTREME_TEMPERATURES',
  'EXTREME_PRESSURE',
  'DIVERSE_LIFE',
  'SCARCE_LIFE',
  'FOSSILS',
  'WEAK_GRAVITY',
  'STRONG_GRAVITY',
  'CRUSHING_GRAVITY',
  'TOXIC_ATMOSPHERE',
  'CORROSIVE_ATMOSPHERE',
  'BREATHABLE_ATMOSPHERE',
  'THIN_ATMOSPHERE',
  'JOVIAN',
  'ROCKY',
  'VOLCANIC',
  'FROZEN',
  'SWAMP',
  'BARREN',
  'TEMPERATE',
  'JUNGLE',
  'OCEAN',
  'RADIOACTIVE',
  'MICRO_GRAVITY_ANOMALIES',
  'DEBRIS_CLUSTER',
  'DEEP_CRATERS',
  'SHALLOW_CRATERS',
  'UNSTABLE_COMPOSITION',
  'HOLLOWED_INTERIOR',
  'STRIPPED'
);
-- is_under_construction
-- orbitals
-- orbits
-- faction
-- modifiers
CREATE TYPE waypoint_modifier_symbol AS ENUM (
  'STRIPPED',
  'UNSTABLE',
  'RADIATION_LEAK',
  'CRITICAL_LIMIT',
  'CIVIL_UNREST'
);
-- chart
ALTER TABLE IF EXISTS public.waypoint
ADD COLUMN x integer NOT NULL,
  ADD COLUMN y integer NOT NULL,
  ADD COLUMN type waypoint_type NOT NULL,
  ADD COLUMN traits waypoint_trait_symbol [] NOT NULL,
  ADD COLUMN is_under_construction boolean NOT NULL,
  ADD COLUMN orbitals character varying [] NOT NULL,
  ADD COLUMN orbits character varying,
  ADD COLUMN faction character varying,
  ADD COLUMN modifiers waypoint_modifier_symbol [] NOT NULL,
  ADD COLUMN charted_by character varying,
  ADD COLUMN charted_on character varying;