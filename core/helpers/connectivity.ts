"use server";

import { newServerClient } from "../supabase/server";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import {
  IConnectivityWithCoordinates,
  IConnectivity,
  ConnectivityInsert,
  ConnectivityUpdate,
} from "../types/db";
import { SupabaseClient } from "@supabase/supabase-js";

// Get connectivity by session id using RPC function with coordinates
export async function server_get_connectivity_by_session_id(
  sessionId: number,
): Promise<IWebResponseCompatible<IConnectivityWithCoordinates[]>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase.rpc(
    "get_connectivity_with_coordinates",
    { session_id_caller: sessionId },
  );

  if (error) {
    console.warn("Error fetching connectivity by session id:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  // Sort by timestamp_start in ascending order
  const sortedConnectivity = (data || []).sort((a, b) => {
    if (!a.timestamp_start || !b.timestamp_start) return 0;
    return (
      new Date(a.timestamp_start).getTime() -
      new Date(b.timestamp_start).getTime()
    );
  });

  return IWebResponse.success(sortedConnectivity).to_compatible();
}

// Get all connectivity items after a specific timestamp, filtered by device ID
// Timestamp should be formatted as YYYY-MM-DDTHH:mm:ss.SSSZ
export async function server_get_connectivity_by_device_id(
  deviceId: number,
  timestamp: string,
): Promise<IWebResponseCompatible<IConnectivityWithCoordinates[]>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase.rpc(
    "get_connectivity_with_coordinates_by_device_and_timestamp",
    { device_id_caller: deviceId, timestamp_filter: timestamp },
  );

  if (error) {
    console.warn("Error fetching connectivity by session id:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  // Sort by timestamp_start in ascending order
  const sortedConnectivity = (data || []).sort((a, b) => {
    if (!a.timestamp_start || !b.timestamp_start) return 0;
    return (
      new Date(a.timestamp_start).getTime() -
      new Date(b.timestamp_start).getTime()
    );
  });

  return IWebResponse.success(sortedConnectivity).to_compatible();
}

// Insert new connectivity records (accepts array for batch operations)
export async function server_insert_connectivity(
  connectivity: ConnectivityInsert | ConnectivityInsert[],
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IConnectivity[]>> {
  const supabase = client || (await newServerClient());

  const connectivityArray = Array.isArray(connectivity)
    ? connectivity
    : [connectivity];

  const { data, error } = await supabase
    .from("connectivity")
    .insert(connectivityArray)
    .select("*");

  if (error) {
    console.warn("Error inserting connectivity:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  return IWebResponse.success(data || []).to_compatible();
}

// Update existing connectivity records (accepts array for batch operations)
// Each connectivity record in the array must include an 'id' field
export async function server_update_connectivity(
  connectivity: (ConnectivityUpdate & { id: number }) | (ConnectivityUpdate & { id: number })[],
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IConnectivity[]>> {
  const supabase = client || (await newServerClient());

  const connectivityArray = Array.isArray(connectivity)
    ? connectivity
    : [connectivity];
  const updatedConnectivity: IConnectivity[] = [];

  for (const record of connectivityArray) {
    const { id, ...updateData } = record;
    // Remove fields that shouldn't be updated
    delete (updateData as any).inserted_at;

    const { data, error } = await supabase
      .from("connectivity")
      .update(updateData)
      .eq("id", id)
      .select("*")
      .single();

    if (error) {
      console.warn("Error updating connectivity:", error.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: [],
      };
    }

    if (data) {
      updatedConnectivity.push(data);
    }
  }

  return IWebResponse.success(updatedConnectivity).to_compatible();
}
