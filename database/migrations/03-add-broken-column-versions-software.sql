-- Migration: Add broken column to versions_software table
-- When broken is true, pre, stable, and min must be false

-- Step 1: Add the broken column
ALTER TABLE "public"."versions_software"
ADD COLUMN "broken" boolean DEFAULT false NOT NULL;

-- Step 2: Create function to enforce broken constraint
CREATE OR REPLACE FUNCTION "private"."enforce_broken_versions_software"()
RETURNS TRIGGER
LANGUAGE "plpgsql"
AS $$
BEGIN
  -- If broken is true, set all other boolean flags to false
  IF NEW.broken = true THEN
    NEW.pre = false;
    NEW.stable = false;
    NEW.min = false;
  END IF;
  RETURN NEW;
END;
$$;

ALTER FUNCTION "private"."enforce_broken_versions_software"() OWNER TO "postgres";

COMMENT ON FUNCTION "private"."enforce_broken_versions_software"() IS 'Ensures that when broken is true, pre, stable, and min are set to false';

-- Step 3: Create trigger to enforce the constraint
CREATE TRIGGER "enforce_broken_versions_software_trigger"
  BEFORE INSERT OR UPDATE ON "public"."versions_software"
  FOR EACH ROW
  EXECUTE FUNCTION "private"."enforce_broken_versions_software"();
