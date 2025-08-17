-- Add up migration script here
CREATE TYPE fund_status AS ENUM ('RESERVED', 'USED', 'CANCELLED');
CREATE TABLE public.reserved_funds (
  id bigserial NOT NULL,
  amount bigint NOT NULL,
  status fund_status NOT NULL,
  actual_amount bigint NOT NULL,
  created_at timestamp with time zone DEFAULT now () NOT NULL,
  updated_at timestamp with time zone DEFAULT now () NOT NULL,
  PRIMARY KEY (id)
);
--
ALTER TABLE IF EXISTS public.contract
ADD COLUMN reserved_fund bigint;
ALTER TABLE IF EXISTS public.contract
ADD CONSTRAINT reserved_fund_relation FOREIGN KEY (reserved_fund) REFERENCES public.reserved_funds (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION;
--
ALTER TABLE IF EXISTS public.construction_shipment
ADD COLUMN reserved_fund bigint;
ALTER TABLE IF EXISTS public.construction_shipment
ADD CONSTRAINT reserved_fund_relation FOREIGN KEY (reserved_fund) REFERENCES public.reserved_funds (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION;
--
ALTER TABLE IF EXISTS public.trade_route
ADD COLUMN reserved_fund bigint;
ALTER TABLE IF EXISTS public.trade_route
ADD CONSTRAINT reserved_fund_relation FOREIGN KEY (reserved_fund) REFERENCES public.reserved_funds (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION;
--
ALTER TABLE IF EXISTS public.ship_transfers
ADD COLUMN reserved_fund bigint;
ALTER TABLE IF EXISTS public.ship_transfers
ADD CONSTRAINT reserved_fund_relation FOREIGN KEY (reserved_fund) REFERENCES public.reserved_funds (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION;