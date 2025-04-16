-- Add down migration script here
DROP TABLE public.ship_transfers;
ALTER TYPE public.ship_info_role DROP VALUE 'Transfer';