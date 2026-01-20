-- Migration: Allow Approved Gateway Device Types to Upload Data for Same-Herd Devices

-- Step 1: Create function to check if a device type is approved for gateway operations
CREATE OR REPLACE FUNCTION "private"."is_approved_gateway_device_type"("device_id_caller" bigint) RETURNS boolean
    LANGUAGE "plpgsql" SECURITY DEFINER
    AS $$
BEGIN
  RETURN EXISTS (
    SELECT 1
    FROM public.devices
    WHERE id = device_id_caller
    AND device_type IN ('radio_mesh_base_station_gateway', 'radio_mesh_base_station', 'radio_mesh_repeater')
  );
END;
$$;

ALTER FUNCTION "private"."is_approved_gateway_device_type"("device_id_caller" bigint) OWNER TO "postgres";

COMMENT ON FUNCTION "private"."is_approved_gateway_device_type"("device_id_caller" bigint) IS 'Returns true if the specified device is of an approved gateway device type';

-- Step 2: Update Sessions policies (INSERT and UPDATE)
-- Drop existing policies
DROP POLICY IF EXISTS "Session creation: Device API keys and users with edit role" ON "public"."sessions";
DROP POLICY IF EXISTS "Session modification: Device API keys and users with edit role" ON "public"."sessions";

-- Recreate Session creation policy with gateway device support
CREATE POLICY "Session creation: Device API keys and users with edit role" ON "public"."sessions" FOR INSERT WITH CHECK (
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
    (SELECT "private"."get_herd_id_by_device_id"("sessions"."device_id") AS "get_herd_id_by_device_id")
  )
);

-- Recreate Session modification policy with gateway device support
CREATE POLICY "Session modification: Device API keys and users with edit role" ON "public"."sessions" FOR UPDATE USING (
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
    (SELECT "private"."get_herd_id_by_device_id"("sessions"."device_id") AS "get_herd_id_by_device_id")
  )
);

-- Step 3: Update Events policies (INSERT and UPDATE)
-- Drop existing policies
DROP POLICY IF EXISTS "Event creation: Device API keys and users with edit role" ON "public"."events";
DROP POLICY IF EXISTS "Event modification: Device API keys and users with edit role" ON "public"."events";

-- Recreate Event creation policy with gateway device support
CREATE POLICY "Event creation: Device API keys and users with edit role" ON "public"."events" FOR INSERT WITH CHECK (
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
    (SELECT "private"."get_herd_id_by_device_id"("events"."device_id") AS "get_herd_id_by_device_id")
  )
);

-- Recreate Event modification policy with gateway device support
CREATE POLICY "Event modification: Device API keys and users with edit role" ON "public"."events" FOR UPDATE USING (
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
    (SELECT "private"."get_herd_id_by_device_id"("events"."device_id") AS "get_herd_id_by_device_id")
  )
);

-- Step 4: Update Connectivity policies (SELECT, INSERT, and UPDATE)
-- Drop existing policies
DROP POLICY IF EXISTS "Connectivity access: Device API keys and users with view role" ON "public"."connectivity";
DROP POLICY IF EXISTS "Connectivity creation: Device API keys and users with edit role" ON "public"."connectivity";
DROP POLICY IF EXISTS "Connectivity modification: Device API keys and users with edit " ON "public"."connectivity";

-- Recreate Connectivity access policy with approved gateway device support
CREATE POLICY "Connectivity access: Device API keys and users with view role" ON "public"."connectivity" FOR SELECT USING (
  (
    ("device_id" IS NULL) 
    AND ("session_id" IS NOT NULL) 
    AND (
      ("session_id" IN (
        SELECT "sessions"."id"
        FROM "public"."sessions"
        WHERE ("sessions"."device_id" = "private"."key_uid"())
      ))
      OR
      (
        ("private"."key_uid"() <> 0) 
        AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
        AND "private"."herd_has_device"(
          "private"."get_herd_id_by_device_id"("private"."key_uid"()), 
          (SELECT "sessions"."device_id"
           FROM "public"."sessions"
           WHERE ("sessions"."id" = "connectivity"."session_id"))
        )
      )
    )
  ) 
  OR 
  (
    ("session_id" IS NULL) 
    AND ("device_id" IS NOT NULL) 
    AND ("device_id" = "private"."key_uid"())
  ) 
  OR 
  (
    ("session_id" IS NOT NULL) 
    AND ("device_id" IS NOT NULL) 
    AND ("device_id" = "private"."key_uid"()) 
    AND ("session_id" IN (
      SELECT "sessions"."id"
      FROM "public"."sessions"
      WHERE ("sessions"."device_id" = "private"."key_uid"())
    ))
  ) 
  OR 
  (
    ("device_id" IS NOT NULL) 
    AND ("device_id" <> "private"."key_uid"()) 
    AND ("private"."key_uid"() <> 0) 
    AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
    AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")
  ) 
  OR 
  "private"."has_good_view_role"(
    (SELECT "auth"."uid"() AS "uid"),
    CASE
      WHEN ("device_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"("device_id")
      WHEN ("session_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"(
        (SELECT "sessions"."device_id"
         FROM "public"."sessions"
         WHERE ("sessions"."id" = "connectivity"."session_id"))
      )
      ELSE NULL::bigint
    END
  )
);


-- Recreate Connectivity creation policy with approved gateway device support
CREATE POLICY "Connectivity creation: Device API keys and users with edit role" ON "public"."connectivity" FOR INSERT WITH CHECK (
  (
    ("device_id" IS NULL) 
    AND ("session_id" IS NOT NULL) 
    AND (
      ("session_id" IN (
        SELECT "sessions"."id"
        FROM "public"."sessions"
        WHERE ("sessions"."device_id" = "private"."key_uid"())
      ))
      OR
      (
        ("private"."key_uid"() <> 0) 
        AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
        AND "private"."herd_has_device"(
          "private"."get_herd_id_by_device_id"("private"."key_uid"()), 
          (SELECT "sessions"."device_id"
           FROM "public"."sessions"
           WHERE ("sessions"."id" = "connectivity"."session_id"))
        )
      )
    )
  ) 
  OR 
  (
    ("session_id" IS NULL) 
    AND ("device_id" IS NOT NULL) 
    AND ("device_id" = "private"."key_uid"())
  ) 
  OR 
  (
    ("session_id" IS NOT NULL) 
    AND ("device_id" IS NOT NULL) 
    AND ("device_id" = "private"."key_uid"()) 
    AND ("session_id" IN (
      SELECT "sessions"."id"
      FROM "public"."sessions"
      WHERE ("sessions"."device_id" = "private"."key_uid"())
    ))
  ) 
  OR 
  (
    ("device_id" IS NOT NULL) 
    AND ("device_id" <> "private"."key_uid"()) 
    AND ("private"."key_uid"() <> 0) 
    AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
    AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")
  ) 
  OR 
  "private"."has_good_edit_role"(
    (SELECT "auth"."uid"() AS "uid"),
    CASE
      WHEN ("device_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"("device_id")
      WHEN ("session_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"(
        (SELECT "sessions"."device_id"
         FROM "public"."sessions"
         WHERE ("sessions"."id" = "connectivity"."session_id"))
      )
      ELSE NULL::bigint
    END
  )
);

-- Recreate Connectivity modification policy with approved gateway device support
CREATE POLICY "Connectivity modification: Device API keys and users with edit " ON "public"."connectivity" FOR UPDATE USING (
  (
    ("device_id" IS NULL) 
    AND ("session_id" IS NOT NULL) 
    AND (
      ("session_id" IN (
        SELECT "sessions"."id"
        FROM "public"."sessions"
        WHERE ("sessions"."device_id" = "private"."key_uid"())
      ))
      OR
      (
        ("private"."key_uid"() <> 0) 
        AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
        AND "private"."herd_has_device"(
          "private"."get_herd_id_by_device_id"("private"."key_uid"()), 
          (SELECT "sessions"."device_id"
           FROM "public"."sessions"
           WHERE ("sessions"."id" = "connectivity"."session_id"))
        )
      )
    )
  ) 
  OR 
  (
    ("session_id" IS NULL) 
    AND ("device_id" IS NOT NULL) 
    AND ("device_id" = "private"."key_uid"())
  ) 
  OR 
  (
    ("session_id" IS NOT NULL) 
    AND ("device_id" IS NOT NULL) 
    AND ("device_id" = "private"."key_uid"()) 
    AND ("session_id" IN (
      SELECT "sessions"."id"
      FROM "public"."sessions"
      WHERE ("sessions"."device_id" = "private"."key_uid"())
    ))
  ) 
  OR 
  (
    ("device_id" IS NOT NULL) 
    AND ("device_id" <> "private"."key_uid"()) 
    AND ("private"."key_uid"() <> 0) 
    AND "private"."is_approved_gateway_device_type"("private"."key_uid"()) 
    AND "private"."herd_has_device"("private"."get_herd_id_by_device_id"("private"."key_uid"()), "device_id")
  ) 
  OR 
  "private"."has_good_edit_role"(
    (SELECT "auth"."uid"() AS "uid"),
    CASE
      WHEN ("device_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"("device_id")
      WHEN ("session_id" IS NOT NULL) THEN "private"."get_herd_id_by_device_id"(
        (SELECT "sessions"."device_id"
         FROM "public"."sessions"
         WHERE ("sessions"."id" = "connectivity"."session_id"))
      )
      ELSE NULL::bigint
    END
  )
);

