-- Create construction_mode enum type
CREATE TYPE construction_mode AS ENUM (
    'LowestPurchaseCost',
    'LowestAbsoluteProgress',
    'LowestPercentProgress',
    'BestPurchaseSupply'
);
-- Add construction_mode column to fleet
ALTER TABLE IF EXISTS public.fleet
ADD COLUMN construction_mode construction_mode;