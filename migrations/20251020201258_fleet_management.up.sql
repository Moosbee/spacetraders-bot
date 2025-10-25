-- Add up migration script here
ALTER TABLE IF EXISTS public.ship_info
ADD COLUMN purchase_id bigint;
ALTER TABLE IF EXISTS public.ship_info
ADD COLUMN assignment_id bigint;
-- Create trade_mode enum type
CREATE TYPE trade_mode AS ENUM (
  'ProfitPerHour',
  'ProfitPerAPIRequest',
  'ProfitPerTrip'
);
-- Create fleet_type enum type
CREATE TYPE fleet_type AS ENUM (
  'Trading',
  'Scrapping',
  'Mining',
  'Charting',
  'Construction',
  'Manuel',
  'Contract'
);
CREATE TABLE fleet (
  id SERIAL PRIMARY KEY,
  system_symbol character varying NOT NULL,
  fleet_type fleet_type NOT NULL,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  -- Trading config
  market_blacklist trade_symbol [],
  market_prefer_list trade_symbol [],
  purchase_multiplier FLOAT8,
  ship_market_ratio FLOAT8,
  min_cargo_space INTEGER,
  trade_mode trade_mode,
  trade_profit_threshold INTEGER,
  -- Scrapping config
  allowed_requests INTEGER,
  notify_on_shipyard BOOLEAN,
  -- Mining config
  mining_eject_list trade_symbol [],
  mining_prefer_list trade_symbol [],
  ignore_engineered_asteroids BOOLEAN,
  stop_all_unstable BOOLEAN,
  mining_waypoints INTEGER,
  unstable_since_timeout INTEGER,
  syphon_waypoints INTEGER,
  miners_per_waypoint INTEGER,
  siphoners_per_waypoint INTEGER,
  surveyors_per_waypoint INTEGER,
  mining_transporter_count INTEGER,
  -- Charting config
  charting_probe_count INTEGER,
  -- Construction config
  construction_ship_count INTEGER,
  construction_waypoint character varying,
  -- Contract config
  contract_ship_count INTEGER
);