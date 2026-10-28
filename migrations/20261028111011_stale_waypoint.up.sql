ALTER TABLE IF EXISTS public.waypoint
ADD COLUMN stale_market integer NOT NULL DEFAULT 0;

ALTER TABLE IF EXISTS public.waypoint
ADD COLUMN stale_shipyard integer NOT NULL DEFAULT 0;
