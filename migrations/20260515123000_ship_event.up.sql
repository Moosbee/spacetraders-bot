CREATE TABLE public.ship_event (
  id bigserial NOT NULL,
  ship_symbol character varying NOT NULL,
  event_kind character varying NOT NULL,
  event_name character varying NOT NULL,
  event_phase character varying NOT NULL,
  correlation_id character varying NOT NULL,
  payload jsonb NOT NULL,
  before_ship_state_id bigint NOT NULL,
  after_ship_state_id bigint NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  PRIMARY KEY (id),
  CONSTRAINT ship_event_ship_symbol_fk FOREIGN KEY (ship_symbol) REFERENCES public.ship_info (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT ship_event_before_ship_state_fk FOREIGN KEY (before_ship_state_id) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT ship_event_after_ship_state_fk FOREIGN KEY (after_ship_state_id) REFERENCES public.ship_state (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION
);

CREATE INDEX ship_event_ship_symbol_created_at_idx
ON public.ship_event (ship_symbol, created_at, id);

CREATE INDEX ship_event_correlation_id_idx
ON public.ship_event (correlation_id);
