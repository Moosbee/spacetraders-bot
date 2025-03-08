-- Add up migration script here
-- Table: public.route
-- DROP TABLE IF EXISTS public.route;
CREATE TABLE IF NOT EXISTS public.route (
  id SERIAL PRIMARY KEY,
  ship_symbol character varying NOT NULL,
  "from" character varying NOT NULL,
  "to" character varying NOT NULL,
  distance double precision NOT NULL,
  nav_mode character varying NOT NULL,
  fuel_cost integer NOT NULL,
  travel_time double precision NOT NULL,
  ship_info_before bigint NOT NULL,
  ship_info_after bigint NOT NULL,
  created_at timestamp without time zone NOT NULL DEFAULT now(),
  CONSTRAINT route_relation_1 FOREIGN KEY ("from") REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT route_relation_2 FOREIGN KEY ("to") REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT route_relation_ship_info_before_fk FOREIGN KEY (ship_info_before) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT route_relation_ship_info_after_fk FOREIGN KEY (ship_info_after) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT route_relation_ship_symbol FOREIGN KEY (ship_symbol) REFERENCES public.ship_info (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
);