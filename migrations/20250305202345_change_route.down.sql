-- Add down migration script here
ALTER TABLE IF EXISTS public.route DROP COLUMN IF EXISTS ship_info_before;
ALTER TABLE IF EXISTS public.route DROP COLUMN IF EXISTS ship_info_after;
ALTER TABLE IF EXISTS public.route DROP COLUMN IF EXISTS created_at;