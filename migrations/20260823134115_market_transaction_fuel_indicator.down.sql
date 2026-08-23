-- Add down migration script here
ALTER TABLE IF EXISTS public.market_transaction DROP COLUMN IF EXISTS is_fuel;