-- Add up migration script here
ALTER TABLE IF EXISTS public.waypoint
ADD COLUMN has_marketplace boolean NOT NULL DEFAULT False;
ALTER TABLE IF EXISTS public.waypoint
ADD COLUMN has_shipyard boolean NOT NULL DEFAULT False;