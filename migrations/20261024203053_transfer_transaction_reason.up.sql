-- Add up migration script here
ALTER TABLE IF EXISTS public.market_transaction
ADD COLUMN IF NOT EXISTS ship_transfer bigint;

ALTER TABLE IF EXISTS public.market_transaction
ADD CONSTRAINT market_transaction_relation_6 FOREIGN KEY (ship_transfer) REFERENCES public.ship_transfers (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION;
