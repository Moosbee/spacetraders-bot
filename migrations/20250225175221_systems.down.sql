-- Add down migration script here
-- Drop foreign key constraint
ALTER TABLE IF EXISTS public.waypoint DROP CONSTRAINT IF EXISTS system_waypoint_relation;
-- Drop index
DROP INDEX IF EXISTS fki_system_waypoint_relation;
-- Drop ENUM types
DROP TYPE IF EXISTS system_type;
-- Drop system table
DROP TABLE IF EXISTS system;