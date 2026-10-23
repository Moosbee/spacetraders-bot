-- Add up migration script here
CREATE TYPE ship_purchase_reservation_state AS ENUM (
  'RESERVED',
  'USED',
  'CANCELLED'
);

CREATE TABLE public.ship_purchase_reservations (
  id bigserial NOT NULL,
  shipyard_ship_id bigint NOT NULL,
  assignment_id bigint NOT NULL,
  fleet_id integer NOT NULL,
  reservation_id bigint NOT NULL,
  state ship_purchase_reservation_state NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id),
  CONSTRAINT fk_ship_purchase_reservation_shipyard_ship FOREIGN KEY (shipyard_ship_id) REFERENCES public.shipyard_ship (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT fk_ship_purchase_reservation_assignment FOREIGN KEY (assignment_id) REFERENCES public.ship_assignment (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT fk_ship_purchase_reservation_fleet FOREIGN KEY (fleet_id) REFERENCES public.fleet (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT fk_ship_purchase_reservation_reserved_fund FOREIGN KEY (reservation_id) REFERENCES public.reserved_funds (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);
