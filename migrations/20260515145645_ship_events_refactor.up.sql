-- Drop tables that reference ship_state (will be replaced by events)
DROP TABLE IF EXISTS public.route CASCADE;
DROP TABLE IF EXISTS public.extraction CASCADE;
DROP TABLE IF EXISTS public.ship_jump CASCADE;
DROP TABLE IF EXISTS public.survey CASCADE;

-- Recreate ship_state with status and auto_pilot as JSON
DROP TABLE IF EXISTS public.ship_state CASCADE;

CREATE TABLE public.ship_state (
  id bigserial NOT NULL,
  symbol character varying NOT NULL,
  display_name character varying NOT NULL,
  engine_speed integer NOT NULL,
  engine_condition double precision NOT NULL,
  engine_integrity double precision NOT NULL,
  frame_condition double precision NOT NULL,
  frame_integrity double precision NOT NULL,
  reactor_condition double precision NOT NULL,
  reactor_integrity double precision NOT NULL,
  fuel_capacity integer NOT NULL,
  fuel_current integer NOT NULL,
  cargo_capacity integer NOT NULL,
  cargo_units integer NOT NULL,
  cargo_inventory jsonb NOT NULL,
  mounts ship_mount_symbol [] NOT NULL,
  modules ship_module_symbol [] NOT NULL,
  reactor_symbol ship_reactor_symbol NOT NULL,
  frame_symbol ship_frame_symbol NOT NULL,
  engine_symbol ship_engine_symbol NOT NULL,
  cooldown_expiration timestamp with time zone,
  cooldown integer,
  flight_mode character varying NOT NULL,
  nav_status character varying NOT NULL,
  system_symbol character varying NOT NULL,
  waypoint_symbol character varying NOT NULL,
  route_arrival timestamp with time zone NOT NULL,
  route_departure timestamp with time zone NOT NULL,
  route_destination_symbol character varying NOT NULL,
  route_destination_system character varying NOT NULL,
  route_origin_symbol character varying NOT NULL,
  route_origin_system character varying NOT NULL,
  status jsonb NOT NULL DEFAULT '{}'::jsonb,
  auto_pilot jsonb,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id)
);

CREATE INDEX idx_ship_state_symbol ON public.ship_state (symbol);
CREATE INDEX idx_ship_state_waypoint ON public.ship_state (waypoint_symbol);
CREATE INDEX idx_ship_state_system ON public.ship_state (system_symbol);
CREATE INDEX idx_ship_state_created_at ON public.ship_state (created_at);

-- Create ship_events table
CREATE TABLE public.ship_event (
  id bigserial NOT NULL,
  ship_symbol character varying NOT NULL,
  event_type character varying NOT NULL,
  event_data jsonb NOT NULL DEFAULT '{}'::jsonb,
  state_before jsonb NOT NULL,
  state_after jsonb,
  duration_ms bigint,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id)
);

CREATE INDEX idx_ship_event_symbol ON public.ship_event (ship_symbol);
CREATE INDEX idx_ship_event_type ON public.ship_event (event_type);
CREATE INDEX idx_ship_event_created_at ON public.ship_event (created_at);
CREATE INDEX idx_ship_event_symbol_type ON public.ship_event (ship_symbol, event_type);
