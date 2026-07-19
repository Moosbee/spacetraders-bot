-- Add down migration script here
DROP INDEX idx_key_values;
DROP TABLE IF EXISTS configuration;