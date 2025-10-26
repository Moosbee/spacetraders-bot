-- Add up migration script here
CREATE TABLE public.ship_assignment (
  id bigserial NOT NULL,
  fleet_id integer NOT NULL,
  priority integer NOT NULL DEFAULT 0,
  disabled boolean NOT NULL DEFAULT false,
  range_min integer NOT NULL DEFAULT 0,
  cargo_min integer NOT NULL DEFAULT 0,
  survey boolean NOT NULL DEFAULT false,
  extractor boolean NOT NULL DEFAULT false,
  siphon boolean NOT NULL DEFAULT false,
  warp_drive boolean NOT NULL DEFAULT false,
  PRIMARY KEY (id),
  CONSTRAINT fk_fleet FOREIGN KEY (fleet_id) REFERENCES public.fleet (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);