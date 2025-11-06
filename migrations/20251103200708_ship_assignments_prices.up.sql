-- Add up migration script here
ALTER TABLE IF EXISTS public.ship_assignment
ADD COLUMN max_purchase_price integer NOT NULL DEFAULT 1000000;
ALTER TABLE IF EXISTS public.ship_assignment
ADD COLUMN credits_threshold integer NOT NULL DEFAULT 500000;