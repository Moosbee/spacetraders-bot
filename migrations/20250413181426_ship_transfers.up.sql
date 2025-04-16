-- Add up migration script here
CREATE TABLE public.ship_transfers (
  id bigserial NOT NULL,
  ship_symbol character varying NOT NULL,
  system_symbol character varying NOT NULL,
  role ship_info_role NOT NULL,
  finished boolean NOT NULL,
  PRIMARY KEY (id)
);
ALTER TYPE public.ship_info_role
ADD VALUE 'Transfer'
AFTER 'Manuel';