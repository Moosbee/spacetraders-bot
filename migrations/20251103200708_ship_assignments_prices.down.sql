-- Add down migration script here
ALTER TABLE IF EXISTS public.ship_assignment DROP COLUMN max_purchase_price;
ALTER TABLE IF EXISTS public.ship_assignment DROP COLUMN credits_threshold;