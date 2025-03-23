-- Add down migration script here
ALTER TYPE public.trade_symbol DROP VALUE 'SHIP_BULK_FREIGHTER';
ALTER TYPE public.trade_symbol DROP VALUE 'FRAME_BULK_FREIGHTER';
ALTER TYPE public.ship_type DROP VALUE 'SHIP_BULK_FREIGHTER';
ALTER TYPE public.ship_frame_symbol DROP VALUE 'FRAME_BULK_FREIGHTER';