-- Add up migration script here
CREATE TABLE public.construction_material (
  id bigserial NOT NULL,
  waypoint_symbol character varying NOT NULL,
  trade_symbol trade_symbol NOT NULL,
  required integer NOT NULL DEFAULT 0,
  fulfilled integer NOT NULL DEFAULT 0,
  created_at timestamp without time zone NOT NULL DEFAULT now(),
  updated_at timestamp without time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id),
  CONSTRAINT waypoint_trade UNIQUE (waypoint_symbol, trade_symbol),
  CONSTRAINT waypoint FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);
CREATE TABLE public.construction_shipment (
  id bigserial NOT NULL,
  material_id bigint NOT NULL,
  construction_site_waypoint character varying NOT NULL,
  ship_symbol character varying NOT NULL,
  trade_symbol trade_symbol NOT NULL,
  units integer NOT NULL,
  purchase_waypoint character varying NOT NULL,
  created_at timestamp without time zone NOT NULL DEFAULT now(),
  updated_at timestamp without time zone NOT NULL DEFAULT now(),
  status shipment_status NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (construction_site_waypoint) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  FOREIGN KEY (purchase_waypoint) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  FOREIGN KEY (ship_symbol) REFERENCES public.ship_info (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  FOREIGN KEY (material_id) REFERENCES public.construction_material (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);