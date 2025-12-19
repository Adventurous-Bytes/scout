"use server";

import { newServerClient } from "../supabase/server";
import { ISessionWithCoordinates, ISessionUsageOverTime } from "../types/db";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";

// Get session by ID with coordinates
export async function server_get_session_by_id(
  sessionId: number,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<ISessionWithCoordinates | null>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase.rpc("get_session_by_id", {
    session_id_caller: sessionId,
  });

  if (error) {
    console.warn("Error fetching session by id:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  }

  // The RPC returns an array, but we want a single session or null
  const session = data && data.length > 0 ? data[0] : null;
  return IWebResponse.success(session).to_compatible();
}

// Get session usage over time by herd
export async function server_get_session_usage_over_time_by_herd(
  herd_id: number,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<ISessionUsageOverTime>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase.rpc("get_session_usage_over_time", {
    start_date_caller: undefined,
    end_date_caller: undefined,
    device_id_caller: undefined,
    herd_id_caller: herd_id,
  });

  if (error) {
    return IWebResponse.error<ISessionUsageOverTime>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<ISessionUsageOverTime>(
      "No session usage data returned",
    ).to_compatible();
  }

  return IWebResponse.success(
    data as unknown as ISessionUsageOverTime,
  ).to_compatible();
}

// Get session usage over time by device
export async function server_get_session_usage_over_time_by_device(
  device_id: number,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<ISessionUsageOverTime>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase.rpc("get_session_usage_over_time", {
    start_date_caller: undefined,
    end_date_caller: undefined,
    device_id_caller: device_id,
    herd_id_caller: undefined,
  });

  if (error) {
    return IWebResponse.error<ISessionUsageOverTime>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<ISessionUsageOverTime>(
      "No session usage data returned",
    ).to_compatible();
  }

  return IWebResponse.success(
    data as unknown as ISessionUsageOverTime,
  ).to_compatible();
}
