-- Add up migration script here
ALTER TABLE IF EXISTS public.surveys
ADD COLUMN ship_symbol character varying NOT NULL;