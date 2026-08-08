-- Refactor market_trade_good:
-- 1. Remove 'created' column (keep 'created_at')
-- 2. Add autoincrement 'id' as new primary key
-- 3. Old primary key becomes a unique constraint
-- Step 1: Drop the existing primary key constraint
ALTER TABLE public.market_trade_good DROP CONSTRAINT IF EXISTS market_trade_good_pkey;
-- Step 2: Drop the 'created' column
ALTER TABLE public.market_trade_good DROP COLUMN IF EXISTS created;
-- Step 3: Add autoincrement 'id' column as new primary key
ALTER TABLE public.market_trade_good
ADD COLUMN id BIGSERIAL PRIMARY KEY;
-- Step 4: Add unique constraint on the old primary key columns (using created_at instead of created)
ALTER TABLE public.market_trade_good
ADD CONSTRAINT market_trade_good_unique_entry UNIQUE (created_at, symbol, waypoint_symbol);