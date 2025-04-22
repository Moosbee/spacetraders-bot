-- Add up migration script here
CREATE TABLE public.ship_jumps (
  id bigserial NOT NULL,
  ship_symbol character varying NOT NULL,
  "from" character varying NOT NULL,
  "to" character varying NOT NULL,
  distance bigint NOT NULL,
  ship_before bigint NOT NULL,
  ship_after bigint NOT NULL,
  PRIMARY KEY (id),
  CONSTRAINT wp_from FOREIGN KEY ("from") REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT wp_to FOREIGN KEY ("to") REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT ship_before FOREIGN KEY (ship_before) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT ship_after FOREIGN KEY (ship_after) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);