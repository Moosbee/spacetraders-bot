-- Add down migration script here
DROP TABLE IF EXISTS public.shipyard_ship;
DROP TYPE IF EXISTS ship_mount_symbol;
DROP TYPE IF EXISTS ship_module_symbol;
DROP TYPE IF EXISTS ship_engine_symbol;
DROP TYPE IF EXISTS ship_reactor_symbol;
DROP TYPE IF EXISTS ship_frame_symbol;