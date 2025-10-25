-- Add up migration script here
ALTER TABLE ship_state DROP COLUMN role;
ALTER TABLE IF EXISTS public.ship_info
ADD COLUMN temp_assignment_id bigint;
DROP TABLE IF EXISTS ship_transfers;