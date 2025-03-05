-- Add up migration script here
CREATE TABLE public.extraction (
  id bigserial NOT NULL,
  ship_symbol character varying NOT NULL,
  waypoint_symbol character varying NOT NULL,
  ship_info_before bigint NOT NULL,
  ship_info_after bigint NOT NULL,
  siphon boolean NOT NULL,
  yield_symbol trade_symbol NOT NULL,
  yield_units integer NOT NULL,
  created_at timestamp without time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id),
  CONSTRAINT ship_info_before_fk FOREIGN KEY (ship_info_before) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT ship_info_after FOREIGN KEY (ship_info_after) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);