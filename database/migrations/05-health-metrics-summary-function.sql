-- Migration: Health metrics summary function (SECURITY INVOKER, RLS applies)

CREATE OR REPLACE FUNCTION "public"."get_health_metrics_summary"(
  "p_device_id" bigint,
  "p_lookback_minutes" integer DEFAULT 60
)
RETURNS TABLE(
  "metric_name" text,
  "min_value" double precision,
  "max_value" double precision,
  "avg_value" double precision,
  "count" bigint
)
LANGUAGE "plpgsql"
SECURITY INVOKER
SET "search_path" TO ''
AS $$
BEGIN
  IF p_lookback_minutes <= 0 THEN
    RAISE EXCEPTION 'Lookback minutes must be positive, got %', p_lookback_minutes;
  END IF;

  RETURN QUERY
  SELECT
    hm.metric_name,
    min(hm.value)::double precision,
    max(hm.value)::double precision,
    avg(hm.value)::double precision,
    count(*)::bigint
  FROM public.health_metrics hm
  WHERE hm.device_id = p_device_id
    AND hm.timestamp >= (now() - (p_lookback_minutes || ' minutes')::interval)
  GROUP BY hm.metric_name;
END;
$$;

ALTER FUNCTION "public"."get_health_metrics_summary"("p_device_id" bigint, "p_lookback_minutes" integer) OWNER TO "postgres";

COMMENT ON FUNCTION "public"."get_health_metrics_summary"("p_device_id" bigint, "p_lookback_minutes" integer) IS 'Returns per-metric min/max/avg/count for a device over a lookback window. SECURITY INVOKER so RLS on health_metrics applies.';

REVOKE ALL ON FUNCTION "public"."get_health_metrics_summary"("p_device_id" bigint, "p_lookback_minutes" integer) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION "public"."get_health_metrics_summary"("p_device_id" bigint, "p_lookback_minutes" integer) TO "anon";
GRANT EXECUTE ON FUNCTION "public"."get_health_metrics_summary"("p_device_id" bigint, "p_lookback_minutes" integer) TO "authenticated";
GRANT EXECUTE ON FUNCTION "public"."get_health_metrics_summary"("p_device_id" bigint, "p_lookback_minutes" integer) TO "service_role";
