-- Add up migration script here
CREATE TYPE public.survey_size AS ENUM ('SMALL', 'MODERATE', 'LARGE');
CREATE TABLE public.surveys (
  signature character varying NOT NULL,
  ship_info_before bigint NOT NULL,
  ship_info_after bigint NOT NULL,
  waypoint_symbol character varying NOT NULL,
  deposits trade_symbol [] NOT NULL,
  expiration timestamp with time zone NOT NULL,
  size survey_size NOT NULL,
  exhausted_since timestamp with time zone,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (signature),
  CONSTRAINT waypoints FOREIGN KEY (waypoint_symbol) REFERENCES public.waypoint (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT ships_before FOREIGN KEY (ship_info_before) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
  CONSTRAINT ship_after FOREIGN KEY (ship_info_after) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);
ALTER TABLE IF EXISTS public.extraction
ADD COLUMN survey character varying;
ALTER TABLE IF EXISTS public.extraction
ADD CONSTRAINT survey_relation FOREIGN KEY (survey) REFERENCES public.surveys (signature) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION;