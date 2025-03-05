-- Add up migration script here
CREATE TABLE public.engine_info (
  symbol ship_engine_symbol NOT NULL,
  name character varying NOT NULL,
  description character varying NOT NULL,
  speed integer NOT NULL,
  power_required integer,
  crew_required integer,
  slots_required integer,
  PRIMARY KEY (symbol)
);