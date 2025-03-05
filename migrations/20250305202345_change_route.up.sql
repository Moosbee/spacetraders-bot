-- Add up migration script here
ALTER TABLE IF EXISTS public.route
ADD COLUMN ship_info_before bigint;
ALTER TABLE IF EXISTS public.route
ADD COLUMN ship_info_after bigint;
ALTER TABLE IF EXISTS public.route
ADD COLUMN created_at timestamp without time zone NOT NULL DEFAULT now();