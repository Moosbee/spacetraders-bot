-- Add down migration script here
ALTER TABLE IF EXISTS public.fleet DROP COLUMN IF EXISTS priority;
