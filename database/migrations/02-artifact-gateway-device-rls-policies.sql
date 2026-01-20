-- Migration: Add Gateway Device Support to Artifacts RLS Policies

-- Step 1: Update Artifacts policies (SELECT, INSERT, UPDATE, DELETE)
-- Drop existing policies
DROP POLICY IF EXISTS "Artifact access: Device API keys and users with view role" ON "public"."artifacts";
DROP POLICY IF EXISTS "Artifact creation: Device API keys and users with edit role" ON "public"."artifacts";
DROP POLICY IF EXISTS "Artifact modification: Device API keys and users with edit role" ON "public"."artifacts";
DROP POLICY IF EXISTS "Artifact deletion: Device API keys and users with edit role" ON "public"."artifacts";

-- Recreate Artifact access policy with gateway device support
CREATE POLICY "Artifact access: Device API keys and users with view role" ON "public"."artifacts" FOR SELECT USING (
  ("device_id" = "private"."key_uid"()) 
  OR 
  (
    ("private"."key_uid"() <> 0) 
    AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
    AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")
  )
  OR 
  "private"."has_good_view_role"(
    (SELECT "auth"."uid"() AS "uid"), 
    (SELECT "private"."get_herd_id_by_device_id"("artifacts"."device_id") AS "get_herd_id_by_device_id")
  )
);

-- Recreate Artifact creation policy with gateway device support
CREATE POLICY "Artifact creation: Device API keys and users with edit role" ON "public"."artifacts" FOR INSERT WITH CHECK (
  ("device_id" = "private"."key_uid"()) 
  OR 
  (
    ("private"."key_uid"() <> 0) 
    AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
    AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")
  )
  OR 
  "private"."has_good_edit_role"(
    (SELECT "auth"."uid"() AS "uid"), 
    (SELECT "private"."get_herd_id_by_device_id"("artifacts"."device_id") AS "get_herd_id_by_device_id")
  )
);

-- Recreate Artifact modification policy with gateway device support
CREATE POLICY "Artifact modification: Device API keys and users with edit role" ON "public"."artifacts" FOR UPDATE USING (
  ("device_id" = "private"."key_uid"()) 
  OR 
  (
    ("private"."key_uid"() <> 0) 
    AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
    AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")
  )
  OR 
  "private"."has_good_edit_role"(
    (SELECT "auth"."uid"() AS "uid"), 
    (SELECT "private"."get_herd_id_by_device_id"("artifacts"."device_id") AS "get_herd_id_by_device_id")
  )
);

-- Recreate Artifact deletion policy with gateway device support
CREATE POLICY "Artifact deletion: Device API keys and users with edit role" ON "public"."artifacts" FOR DELETE USING (
  ("device_id" = "private"."key_uid"()) 
  OR 
  (
    ("private"."key_uid"() <> 0) 
    AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
    AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")
  )
  OR 
  "private"."has_good_edit_role"(
    (SELECT "auth"."uid"() AS "uid"), 
    (SELECT "private"."get_herd_id_by_device_id"("artifacts"."device_id") AS "get_herd_id_by_device_id")
  )
);
