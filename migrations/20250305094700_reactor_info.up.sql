-- Add up migration script here
CREATE TABLE public.reactor_info (
  symbol ship_reactor_symbol NOT NULL,
  name character varying NOT NULL,
  description character varying NOT NULL,
  power_output integer NOT NULL,
  power_required integer,
  crew_required integer,
  slots_required integer,
  PRIMARY KEY (symbol)
);