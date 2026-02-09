-- Migration: Health metrics table with RLS (device-scoped, same pattern as artifacts/connectivity)

-- Step 1: Create table (device_id for RLS; source optional e.g. hostname)
CREATE TABLE IF NOT EXISTS "public"."health_metrics" (
  "id"          BIGSERIAL PRIMARY KEY,
  "timestamp"   TIMESTAMPTZ NOT NULL,
  "device_id"   BIGINT NOT NULL,
  "source"      TEXT,
  "metric_name" TEXT NOT NULL,
  "value"       DOUBLE PRECISION NOT NULL,
  "unit"        TEXT,
  "created_at"  TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

ALTER TABLE "public"."health_metrics" OWNER TO "postgres";

COMMENT ON TABLE "public"."health_metrics" IS 'Device health metrics (CPU, GPU, memory, storage, etc.). One row per metric per timestamp. New metrics require no schema change.';

ALTER TABLE ONLY "public"."health_metrics"
  ADD CONSTRAINT "health_metrics_device_id_fkey" FOREIGN KEY ("device_id")
  REFERENCES "public"."devices"("id") ON UPDATE CASCADE ON DELETE CASCADE;

CREATE INDEX "idx_health_metrics_lookup" ON "public"."health_metrics"
  USING btree ("device_id", "metric_name", "timestamp" DESC);

-- Step 2: Enable RLS
ALTER TABLE "public"."health_metrics" ENABLE ROW LEVEL SECURITY;

-- Step 3: Policies (same pattern as artifacts: device key, gateway same-herd, user view/edit role)
CREATE POLICY "Health metrics access: Device API keys and users with view role"
  ON "public"."health_metrics" FOR SELECT USING (
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
    (SELECT "private"."get_herd_id_by_device_id"("health_metrics"."device_id") AS "get_herd_id_by_device_id")
  )
);

CREATE POLICY "Health metrics creation: Device API keys and users with edit role"
  ON "public"."health_metrics" FOR INSERT WITH CHECK (
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
    (SELECT "private"."get_herd_id_by_device_id"("health_metrics"."device_id") AS "get_herd_id_by_device_id")
  )
);

CREATE POLICY "Health metrics modification: Device API keys and users with edit role"
  ON "public"."health_metrics" FOR UPDATE USING (
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
    (SELECT "private"."get_herd_id_by_device_id"("health_metrics"."device_id") AS "get_herd_id_by_device_id")
  )
);

CREATE POLICY "Health metrics deletion: Device API keys and users with edit role"
  ON "public"."health_metrics" FOR DELETE USING (
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
    (SELECT "private"."get_herd_id_by_device_id"("health_metrics"."device_id") AS "get_herd_id_by_device_id")
  )
);
