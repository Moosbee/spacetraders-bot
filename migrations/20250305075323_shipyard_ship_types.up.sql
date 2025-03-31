-- Add up migration script here
CREATE TABLE public.shipyard_ship_types (
  id bigserial NOT NULL,
  shipyard_id bigint NOT NULL,
  ship_type ship_type NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id),
  CONSTRAINT shipyard_fk FOREIGN KEY (shipyard_id) REFERENCES public.shipyard (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);