-- Migration: Add path_build_artifact column to versions_software table
-- Path where the corresponding software image file is stored 

ALTER TABLE "public"."versions_software"
ADD COLUMN "path_build_artifact" text;

COMMENT ON COLUMN "public"."versions_software"."path_build_artifact" IS 'Path where the corresponding software image file is stored';
