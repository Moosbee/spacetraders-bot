-- Add up migration script here
CREATE TABLE public.contract_shipment (
  id serial NOT NULL,
  contract_id character varying NOT NULL,
  ship_symbol character varying NOT NULL,
  trade_symbol trade_symbol NOT NULL,
  units integer NOT NULL,
  destination_symbol character varying NOT NULL,
  purchase_symbol character varying NOT NULL,
  created_at timestamp NOT NULL DEFAULT now (),
  updated_at timestamp NOT NULL DEFAULT now (),
  status shipment_status NOT NULL DEFAULT 'IN_TRANSIT',
  PRIMARY KEY (id),
  CONSTRAINT pur_waypoints_dest FOREIGN KEY (purchase_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT dest_waypoints_dest FOREIGN KEY (destination_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT contract_key FOREIGN KEY (contract_id) REFERENCES public.contract (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);