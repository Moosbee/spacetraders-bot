-- Add down migration script here
ALTER TABLE IF EXISTS public.surveys DROP COLUMN ship_symbol;