"use server";

import { newServerClient } from "../supabase/server";

import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";

import { IHeartbeat } from "../types/db";

// Function to get the last heartbeat for a device
export async function server_get_last_heartbeat_by_device(
  device_id: number
): Promise<IWebResponseCompatible<IHeartbeat | null>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase
    .from("heartbeats")
    .select("*")
    .eq("device_id", device_id)
    .order("timestamp", { ascending: false })
    .limit(1)
    .single();

  if (error) {
    // If no heartbeats found, return null data instead of error
    if (error.code === "PGRST116") {
      return {
        status: EnumWebResponse.SUCCESS,
        msg: null,
        data: null,
      };
    }

    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  } else {
    return IWebResponse.success(data).to_compatible();
  }
}

// Function to get all heartbeats for a device
export async function server_get_heartbeats_by_device(
  device_id: number,
  limit?: number
): Promise<IWebResponseCompatible<IHeartbeat[]>> {
  const supabase = await newServerClient();

  let query = supabase
    .from("heartbeats")
    .select("*")
    .eq("device_id", device_id)
    .order("timestamp", { ascending: false });

  if (limit) {
    query = query.limit(limit);
  }

  const { data, error } = await query;

  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  } else {
    return IWebResponse.success(data || []).to_compatible();
  }
}

// Function to check if a device is online based on heartbeat recency
export async function server_check_device_online_status(
  device_id: number,
  offline_threshold_minutes: number = 5
): Promise<IWebResponseCompatible<{
  is_online: boolean;
  last_heartbeat: IHeartbeat | null;
  minutes_since_last_heartbeat: number | null;
}>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase
    .from("heartbeats")
    .select("*")
    .eq("device_id", device_id)
    .order("timestamp", { ascending: false })
    .limit(1)
    .single();

  if (error) {
    // If no heartbeats found
    if (error.code === "PGRST116") {
      return IWebResponse.success({
        is_online: false,
        last_heartbeat: null,
        minutes_since_last_heartbeat: null,
      }).to_compatible();
    }

    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  }

  const now = new Date();
  const heartbeatTime = new Date(data.timestamp);
  const minutesSinceLastHeartbeat = Math.floor(
    (now.getTime() - heartbeatTime.getTime()) / (1000 * 60)
  );

  const isOnline = minutesSinceLastHeartbeat <= offline_threshold_minutes;

  return IWebResponse.success({
    is_online: isOnline,
    last_heartbeat: data,
    minutes_since_last_heartbeat: minutesSinceLastHeartbeat,
  }).to_compatible();
}
