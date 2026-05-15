ALTER TABLE public.ship_state
ADD COLUMN status jsonb NOT NULL,
ADD COLUMN auto_pilot_state jsonb NOT NULL;
