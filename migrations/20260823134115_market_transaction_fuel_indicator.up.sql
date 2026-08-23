-- Add up migration script here
ALTER TABLE IF EXISTS public.market_transaction
ADD COLUMN IF NOT EXISTS is_fuel BOOLEAN NOT NULL DEFAULT FALSE;