-- Add down migration script here
-- First drop tables with foreign key dependencies
DROP TABLE IF EXISTS market_transaction;
DROP TABLE IF EXISTS market_trade;
DROP TABLE IF EXISTS market_trade_good;
DROP TABLE IF EXISTS contract_delivery;
DROP TABLE IF EXISTS trade_route;
DROP TABLE IF EXISTS waypoint;
-- Then drop tables without dependencies
DROP TABLE IF EXISTS system;
DROP TABLE IF EXISTS contract;
DROP TABLE IF EXISTS agent;
DROP TABLE IF EXISTS ship_info;
-- Finally drop the custom types
DROP TYPE IF EXISTS trade_symbol;
DROP TYPE IF EXISTS market_transaction_type;
DROP TYPE IF EXISTS market_trade_good_type;
DROP TYPE IF EXISTS supply_level;
DROP TYPE IF EXISTS activity_level;
DROP TYPE IF EXISTS contract_type;
DROP TYPE IF EXISTS ship_info_role;
DROP TYPE IF EXISTS system_type;