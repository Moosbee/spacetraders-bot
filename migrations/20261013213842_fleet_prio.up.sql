-- Add up migration script here
ALTER TABLE IF EXISTS public.fleet
ADD COLUMN priority integer NOT NULL DEFAULT 1000;