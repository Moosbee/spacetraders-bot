-- Add up migration script here
CREATE TABLE public.frame_info (
  symbol ship_frame_symbol NOT NULL,
  name character varying NOT NULL,
  description character varying NOT NULL,
  module_slots integer NOT NULL,
  mounting_points integer NOT NULL,
  fuel_capacity integer NOT NULL,
  power_required integer,
  crew_required integer,
  slots_required integer,
  PRIMARY KEY (symbol)
);