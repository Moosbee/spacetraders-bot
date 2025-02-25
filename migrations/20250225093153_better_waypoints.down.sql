-- Add down migration script here
-- Drop added columns from waypoint table
ALTER TABLE IF EXISTS public.waypoint DROP COLUMN IF EXISTS x,
  DROP COLUMN IF EXISTS y,
  DROP COLUMN IF EXISTS type,
  DROP COLUMN IF EXISTS traits,
  DROP COLUMN IF EXISTS is_under_construction,
  DROP COLUMN IF EXISTS orbitals,
  DROP COLUMN IF EXISTS orbits,
  DROP COLUMN IF EXISTS faction,
  DROP COLUMN IF EXISTS modifiers,
  DROP COLUMN IF EXISTS charted_by,
  DROP COLUMN IF EXISTS charted_on;
-- Drop ENUM types
DROP TYPE IF EXISTS waypoint_type;
DROP TYPE IF EXISTS waypoint_trait_symbol;
DROP TYPE IF EXISTS waypoint_modifier_symbol;