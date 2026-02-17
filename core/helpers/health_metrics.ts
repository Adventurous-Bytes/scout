"use server";

import { newServerClient } from "../supabase/server";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { IHealthMetric, IHealthMetricSummaryRow } from "../types/db";

export async function server_get_health_metrics(
  device_id: number,
  options: {
    lookbackSeconds: number;
    maxCount: number;
    timestampAnchor?: string | null;
    source?: string | null;
    metricName?: string | null;
  }
): Promise<IWebResponseCompatible<IHealthMetric[]>> {
  const supabase = await newServerClient();
  const lookbackMs = options.lookbackSeconds * 1000;
  const anchorMs =
    options.timestampAnchor != null && options.timestampAnchor !== ""
      ? new Date(options.timestampAnchor).getTime()
      : Date.now();
  const startMs = anchorMs - lookbackMs;
  const startISO = new Date(startMs).toISOString();
  const endISO = new Date(anchorMs).toISOString();

  let query = supabase
    .from("health_metrics")
    .select("*")
    .eq("device_id", device_id)
    .gte("timestamp", startISO)
    .lt("timestamp", endISO)
    .order("timestamp", { ascending: false })
    .limit(options.maxCount);

  if (options.source != null && options.source !== "") {
    query = query.eq("source", options.source);
  }
  if (options.metricName != null && options.metricName !== "") {
    query = query.eq("metric_name", options.metricName);
  }

  const { data, error } = await query;
  if (error) {
    return { status: EnumWebResponse.ERROR, msg: error.message, data: [] };
  }
  return IWebResponse.success(data ?? []).to_compatible();
}

export async function server_get_health_metrics_summary(
  device_id: number,
  lookbackMinutes: number = 60
): Promise<IWebResponseCompatible<IHealthMetricSummaryRow[]>> {
  const supabase = await newServerClient();
  const { data, error } = await supabase.rpc("get_health_metrics_summary", {
    p_device_id: device_id,
    p_lookback_minutes: lookbackMinutes,
  });
  if (error) {
    return { status: EnumWebResponse.ERROR, msg: error.message, data: [] };
  }
  return IWebResponse.success(data ?? []).to_compatible();
}
