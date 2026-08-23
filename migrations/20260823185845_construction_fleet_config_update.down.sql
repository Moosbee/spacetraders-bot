-- Remove construction_mode column from fleet
ALTER TABLE IF EXISTS public.fleet DROP COLUMN IF EXISTS construction_mode;
-- Drop construction_mode enum type
DROP TYPE IF EXISTS construction_mode;