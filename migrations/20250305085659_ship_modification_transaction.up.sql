-- Add up migration script here
CREATE TABLE public.ship_modification_transaction (
  waypoint_symbol character varying NOT NULL,
  ship_symbol character varying NOT NULL,
  trade_symbol trade_symbol NOT NULL,
  total_price integer NOT NULL,
  "timestamp" timestamp without time zone NOT NULL,
  PRIMARY KEY (
    waypoint_symbol,
    ship_symbol,
    trade_symbol,
    "timestamp"
  ),
  CONSTRAINT waypoint_symbol_fk FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT ship_symbol_fk FOREIGN KEY (ship_symbol) REFERENCES public.ship_info (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);