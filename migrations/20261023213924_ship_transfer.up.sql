-- Add up migration script here
CREATE TYPE ship_transfer_state AS ENUM (
  'ON_GOING',
  'COMPLETED'
);

CREATE TABLE public.ship_transfers (
  id bigserial NOT NULL,
  ship_symbol character varying NOT NULL,
  state ship_transfer_state NOT NULL,
  fleet_id integer NOT NULL,
  assignment_id bigint NOT NULL,
  start_system_symbol character varying NOT NULL,
  end_system_symbol character varying NOT NULL,
  reservation_id bigint NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id),
  CONSTRAINT fk_ship_transfer_fleet FOREIGN KEY (fleet_id) REFERENCES public.fleet (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT fk_ship_transfer_assignment FOREIGN KEY (assignment_id) REFERENCES public.ship_assignment (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);
