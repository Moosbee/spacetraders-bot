-- Add down migration script here
ALTER TABLE IF EXISTS public.ship_state
ADD COLUMN active boolean NOT NULL DEFAULT True;