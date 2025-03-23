-- Add down migration script here
ALTER TABLE IF EXISTS public.waypoint DROP COLUMN IF EXISTS has_marketplace;
ALTER TABLE IF EXISTS public.waypoint DROP COLUMN IF EXISTS has_shipyard;