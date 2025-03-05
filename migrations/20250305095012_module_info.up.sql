-- Add up migration script here
CREATE TABLE public.module_info (
  symbol ship_module_symbol NOT NULL,
  name character varying NOT NULL,
  description character varying NOT NULL,
  range integer,
  capacity integer,
  power_required integer,
  crew_required integer,
  slots_required integer,
  PRIMARY KEY (symbol)
);