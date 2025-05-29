-- Add up migration script here
CREATE TABLE public.mount_info (
  symbol ship_mount_symbol NOT NULL,
  name character varying NOT NULL,
  description character varying NOT NULL,
  power_required integer,
  crew_required integer,
  slots_required integer,
  strength integer,
  deposits trade_symbol [],
  PRIMARY KEY (symbol)
);