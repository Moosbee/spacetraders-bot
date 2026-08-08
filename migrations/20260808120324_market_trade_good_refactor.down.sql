-- Revert market_trade_good refactor
-- Step 1: Drop the unique constraint
ALTER TABLE public.market_trade_good DROP CONSTRAINT IF EXISTS market_trade_good_unique_entry;
-- Step 2: Drop the new primary key (id column)
ALTER TABLE public.market_trade_good DROP CONSTRAINT IF EXISTS market_trade_good_pkey;
ALTER TABLE public.market_trade_good DROP COLUMN IF EXISTS id;
-- Step 3: Add back the 'created' column
ALTER TABLE public.market_trade_good
ADD COLUMN created timestamp with time zone NOT NULL DEFAULT now();
-- Step 4: Restore the old primary key
ALTER TABLE public.market_trade_good
ADD CONSTRAINT market_trade_good_pkey PRIMARY KEY (created, symbol, waypoint_symbol);