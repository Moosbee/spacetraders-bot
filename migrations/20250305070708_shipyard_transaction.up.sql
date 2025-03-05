-- Add up migration script here
CREATE TYPE ship_type AS ENUM (
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
CREATE TABLE public.shipyard_transaction (
  waypoint_symbol character varying NOT NULL,
  ship_type character varying NOT NULL,
  price integer NOT NULL,
  agent_symbol character varying NOT NULL,
  "timestamp" timestamp without time zone NOT NULL,
  PRIMARY KEY (
    waypoint_symbol,
    ship_type,
    agent_symbol,
    "timestamp"
  ),
  CONSTRAINT waypoint_fk FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);