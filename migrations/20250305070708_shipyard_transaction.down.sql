-- Add down migration script here
-- Drop ENUM types
DROP TYPE IF EXISTS ship_type;
-- Drop system table
DROP TABLE IF EXISTS shipyard_transaction;