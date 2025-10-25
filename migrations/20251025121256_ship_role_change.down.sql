-- Add down migration script here
ALTER TABLE ship_state
ADD COLUMN role ship_info_role;
ALTER TABLE IF EXISTS public.ship_info DROP COLUMN temp_assignment_id;
CREATE TABLE public.ship_transfers (
  id bigserial NOT NULL,
  ship_symbol character varying NOT NULL,
  system_symbol character varying NOT NULL,
  role ship_info_role NOT NULL,
  finished boolean NOT NULL,
  PRIMARY KEY (id)
);