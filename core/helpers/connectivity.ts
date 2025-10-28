import { newServerClient } from "@/supabase";
import {
  EnumWebResponse,
  IConnectivityWithCoordinates,
  IWebResponse,
  IWebResponseCompatible,
} from "@/types";

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
