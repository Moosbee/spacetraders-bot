-- Add down migration script here
-- Step 1: Drop the new foreign key constraint and column from `extraction`
ALTER TABLE IF EXISTS public.extraction DROP CONSTRAINT IF EXISTS survey_relation;
ALTER TABLE IF EXISTS public.extraction DROP COLUMN IF EXISTS survey;
-- Step 2: Drop the `surveys` table
DROP TABLE IF EXISTS public.surveys;
-- Step 3: Drop the `survey_size` ENUM type
DROP TYPE IF EXISTS public.survey_size;