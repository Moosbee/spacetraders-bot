-- Add down migration script here
ALTER TABLE IF EXISTS public.market_transaction DROP CONSTRAINT IF EXISTS market_transaction_relation_6;
ALTER TABLE IF EXISTS public.market_transaction DROP COLUMN IF EXISTS ship_transfer;
