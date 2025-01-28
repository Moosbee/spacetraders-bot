-- Add down migration script here
DROP TABLE IF EXISTS public.contract_shipment;

DROP TYPE IF EXISTS shipment_status;