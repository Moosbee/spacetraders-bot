-- Add down migration script here
ALTER TABLE IF EXISTS public.system DROP COLUMN IF EXISTS constellation;
ALTER TABLE IF EXISTS public.system DROP COLUMN IF EXISTS population_disabled;