-- Add up migration script here
-- Table: public.market_transaction
-- DROP TABLE IF EXISTS public.market_transaction;
CREATE TABLE IF NOT EXISTS public.market_transaction (
  waypoint_symbol character varying(255) NOT NULL,
  ship_symbol character varying(255) NOT NULL,
  type market_transaction_type NOT NULL,
  units integer NOT NULL,
  price_per_unit integer NOT NULL,
  total_price integer NOT NULL,
  trade_symbol trade_symbol NOT NULL,
  contract character varying(255),
  trade_route integer,
  mining character varying(255),
  construction bigint,
  "timestamp" timestamp without time zone NOT NULL,
  created_at timestamp without time zone NOT NULL DEFAULT now(),
  updated_at timestamp without time zone NOT NULL DEFAULT now(),
  CONSTRAINT market_transaction_pkey PRIMARY KEY (
    waypoint_symbol,
    ship_symbol,
    trade_symbol,
    "timestamp"
  ),
  CONSTRAINT market_transaction_relation_0 FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT market_transaction_relation_1 FOREIGN KEY (ship_symbol) REFERENCES public.ship_info (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT market_transaction_relation_2 FOREIGN KEY (mining) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT market_transaction_relation_3 FOREIGN KEY (contract) REFERENCES public.contract (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT market_transaction_relation_4 FOREIGN KEY (trade_route) REFERENCES public.trade_route (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT market_transaction_relation_5 FOREIGN KEY (construction) REFERENCES public.construction_shipment (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
);