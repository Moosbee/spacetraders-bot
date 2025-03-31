-- Add up migration script here
CREATE TYPE ship_frame_symbol AS ENUM (
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
  'FRAME_CARRIER'
);
CREATE TYPE ship_reactor_symbol AS ENUM (
  'REACTOR_SOLAR_I',
  'REACTOR_FUSION_I',
  'REACTOR_FISSION_I',
  'REACTOR_CHEMICAL_I',
  'REACTOR_ANTIMATTER_I'
);
CREATE TYPE ship_engine_symbol AS ENUM (
  'ENGINE_IMPULSE_DRIVE_I',
  'ENGINE_ION_DRIVE_I',
  'ENGINE_ION_DRIVE_II',
  'ENGINE_HYPER_DRIVE_I'
);
CREATE TYPE ship_module_symbol AS ENUM (
  'MODULE_MINERAL_PROCESSOR_I',
  'MODULE_GAS_PROCESSOR_I',
  'MODULE_CARGO_HOLD_I',
  'MODULE_CARGO_HOLD_II',
  'MODULE_CARGO_HOLD_III',
  'MODULE_CREW_QUARTERS_I',
  'MODULE_ENVOY_QUARTERS_I',
  'MODULE_PASSENGER_CABIN_I',
  'MODULE_MICRO_REFINERY_I',
  'MODULE_ORE_REFINERY_I',
  'MODULE_FUEL_REFINERY_I',
  'MODULE_SCIENCE_LAB_I',
  'MODULE_JUMP_DRIVE_I',
  'MODULE_JUMP_DRIVE_II',
  'MODULE_JUMP_DRIVE_III',
  'MODULE_WARP_DRIVE_I',
  'MODULE_WARP_DRIVE_II',
  'MODULE_WARP_DRIVE_III',
  'MODULE_SHIELD_GENERATOR_I',
  'MODULE_SHIELD_GENERATOR_II'
);
CREATE TYPE ship_mount_symbol AS ENUM (
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
  'MOUNT_TURRET_I'
);
CREATE TABLE public.shipyard_ship (
  id bigserial NOT NULL,
  waypoint_symbol character varying NOT NULL,
  ship_type ship_type NOT NULL,
  name character varying NOT NULL,
  supply supply_level NOT NULL,
  activity activity_level,
  purchase_price integer NOT NULL,
  frame_type ship_frame_symbol NOT NULL,
  frame_quality double precision,
  reactor_type ship_reactor_symbol NOT NULL,
  reactor_quality double precision,
  engine_type ship_engine_symbol NOT NULL,
  engine_quality double precision,
  modules ship_module_symbol [] NOT NULL,
  mounts ship_mount_symbol [] NOT NULL,
  crew_requirement integer NOT NULL,
  crew_capacity integer NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id),
  CONSTRAINT waypoint_symbol_fk FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);