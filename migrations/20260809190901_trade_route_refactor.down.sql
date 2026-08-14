-- Add down migration script here
-- rm trade_mode, fleet_id, assignment_id, purchase_trade_good_id, sell_trade_good_id, estimated_fuel
ALTER TABLE IF EXISTS public.trade_route DROP COLUMN IF EXISTS estimated_fuel;
ALTER TABLE IF EXISTS public.trade_route DROP COLUMN IF EXISTS trade_mode;
ALTER TABLE IF EXISTS public.trade_route DROP COLUMN IF EXISTS fleet_id;
ALTER TABLE IF EXISTS public.trade_route DROP COLUMN IF EXISTS assignment_id;
ALTER TABLE IF EXISTS public.trade_route DROP COLUMN IF EXISTS purchase_trade_good_id;
ALTER TABLE IF EXISTS public.trade_route DROP COLUMN IF EXISTS sell_trade_good_id;
-- add predicted_purchase_price and predicted_sell_price
ALTER TABLE IF EXISTS public.trade_route
ADD COLUMN predicted_purchase_price integer NOT NULL;
ALTER TABLE IF EXISTS public.trade_route
ADD COLUMN predicted_sell_price integer NOT NULL;