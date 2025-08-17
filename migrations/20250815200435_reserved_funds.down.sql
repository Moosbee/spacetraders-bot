-- Add down migration script here
ALTER TABLE IF EXISTS public.trade_route DROP CONSTRAINT IF EXISTS reserved_fund_relation;
ALTER TABLE IF EXISTS public.trade_route DROP COLUMN IF EXISTS reserved_fund;
--

ALTER TABLE IF EXISTS public.construction_shipment DROP CONSTRAINT IF EXISTS reserved_fund_relation;
ALTER TABLE IF EXISTS public.construction_shipment DROP COLUMN IF EXISTS reserved_fund;
--

ALTER TABLE IF EXISTS public.contract DROP CONSTRAINT IF EXISTS reserved_fund_relation;
ALTER TABLE IF EXISTS public.contract DROP COLUMN IF EXISTS reserved_fund;
--

DROP TABLE IF EXISTS public.reserved_funds;
DROP TYPE IF EXISTS fund_status;