-- Add down migration script here
ALTER TABLE IF EXISTS public.waypoint DROP COLUMN IF EXISTS unstable_since;