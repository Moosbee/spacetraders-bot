-- Add up migration script here
ALTER TABLE IF EXISTS public.waypoint
ADD COLUMN unstable_since timestamp with time zone;