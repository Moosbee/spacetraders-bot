-- Add up migration script here
CREATE INDEX waypoint_system_symbol_idx ON public.waypoint USING btree (system_symbol);
CREATE INDEX market_trade_waypoint_symbol_symbol_created_at_idx ON public.market_trade USING btree (waypoint_symbol, symbol, created_at DESC);
CREATE INDEX market_trade_symbol_waypoint_symbol_created_at_idx ON public.market_trade USING btree (symbol, waypoint_symbol, created_at DESC);