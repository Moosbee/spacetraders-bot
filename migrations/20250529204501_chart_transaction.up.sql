-- Add up migration script here
CREATE TABLE public.chart_transaction (
  id bigserial NOT NULL,
  waypoint_symbol character varying NOT NULL,
  ship_symbol character varying NOT NULL,
  total_price integer NOT NULL,
  "timestamp" timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  CONSTRAINT uniqueWp UNIQUE (waypoint_symbol),
  CONSTRAINT wp FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT ship FOREIGN KEY (ship_symbol) REFERENCES public.ship_info (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);