-- Add up migration script here
CREATE INDEX created_index ON public.market_trade_good USING btree (created DESC NULLS LAST) WITH (deduplicate_items = True);
CREATE INDEX waypoint_index ON public.market_trade_good USING hash (waypoint_symbol);
CREATE INDEX symbol_index ON public.market_trade_good USING hash (symbol);
CREATE INDEX created_at_index ON public.market_trade USING btree (created_at) WITH (deduplicate_items = True);
CREATE INDEX waypoint_symbol_index ON public.market_trade USING hash (waypoint_symbol);
CREATE INDEX trade_symbol_index ON public.market_trade USING hash (symbol);