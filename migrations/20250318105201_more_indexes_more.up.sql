-- Add up migration script here
ALTER TYPE public.ship_info_role
ADD VALUE 'Charter'
AFTER 'TempTrader';
CREATE INDEX waypoint_symbol_index_sort ON public.market_trade_good USING btree (waypoint_symbol DESC NULLS LAST) WITH (deduplicate_items = True);
CREATE INDEX symbol_index_sort ON public.market_trade_good USING btree (symbol DESC NULLS LAST) WITH (deduplicate_items = True);