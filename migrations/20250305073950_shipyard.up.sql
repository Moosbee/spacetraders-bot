-- Add up migration script here
CREATE TABLE public.shipyard (
  id bigserial NOT NULL,
  waypoint_symbol character varying NOT NULL,
  modifications_fee integer NOT NULL,
  created_at timestamp without time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id),
  CONSTRAINT waypoint_symbol_fk FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);