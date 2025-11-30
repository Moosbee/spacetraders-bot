-- Add up migration script here
ALTER TABLE IF EXISTS public.system
ADD COLUMN constellation character varying;
ALTER TABLE IF EXISTS public.system
ADD COLUMN population_disabled boolean not null DEFAULT false;