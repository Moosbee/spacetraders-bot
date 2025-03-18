-- Add up migration script here
CREATE TABLE public.jump_gate_connections (
  id bigserial NOT NULL,
  waypoint_from character varying NOT NULL,
  waypoint_to character varying NOT NULL,
  created_at timestamp without time zone NOT NULL DEFAULT now(),
  updated_at timestamp without time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id),
  CONSTRAINT uni UNIQUE (waypoint_from, waypoint_to) -- CONSTRAINT waypoint_to FOREIGN KEY (waypoint_to) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  -- CONSTRAINT waypoint_from FOREIGN KEY (waypoint_from) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);