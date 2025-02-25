-- Add up migration script here
CREATE TYPE system_type AS ENUM (
  'NEUTRON_STAR',
  'RED_STAR',
  'ORANGE_STAR',
  'BLUE_STAR',
  'YOUNG_STAR',
  'WHITE_DWARF',
  'BLACK_HOLE',
  'HYPERGIANT',
  'NEBULA',
  'UNSTABLE'
);
CREATE TABLE system (
  symbol character varying PRIMARY KEY,
  sector_symbol character varying NOT NULL,
  system_type system_type NOT NULL,
  x INTEGER NOT NULL,
  y INTEGER NOT NULL
);
ALTER TABLE IF EXISTS public.waypoint
ADD CONSTRAINT system_waypoint_relation FOREIGN KEY (system_symbol) REFERENCES public.system (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION;
CREATE INDEX IF NOT EXISTS fki_system_waypoint_relation ON public.waypoint(system_symbol);