-- Add down migration script here
ALTER TABLE IF EXISTS public.ship_info DROP COLUMN IF EXISTS purchase_id;
ALTER TABLE IF EXISTS public.ship_info DROP COLUMN IF EXISTS assignment_id;
DROP TABLE IF EXISTS public.fleet;
-- Drop trade_mode enum type
DROP TYPE IF EXISTS trade_mode;
-- Drop fleet_type enum type
DROP TYPE IF EXISTS fleet_type;