-- Add up migration script here
-- rm predicted_purchase_price and predicted_sell_price
ALTER TABLE IF EXISTS public.trade_route DROP COLUMN IF EXISTS predicted_purchase_price;
ALTER TABLE IF EXISTS public.trade_route DROP COLUMN IF EXISTS predicted_sell_price;
-- add trade_mode, fleet_id, assignment_id, purchase_trade_good_id, sell_trade_good_id, estimated_fuel
ALTER TABLE IF EXISTS public.trade_route
ADD COLUMN purchase_trade_good_id bigint;
ALTER TABLE IF EXISTS public.trade_route
ADD COLUMN sell_trade_good_id bigint;
ALTER TABLE IF EXISTS public.trade_route
ADD COLUMN trade_mode trade_mode NOT NULL;
ALTER TABLE IF EXISTS public.trade_route
ADD COLUMN fleet_id integer;
ALTER TABLE IF EXISTS public.trade_route
ADD COLUMN assignment_id bigint;
ALTER TABLE IF EXISTS public.trade_route
ADD COLUMN estimated_fuel integer;