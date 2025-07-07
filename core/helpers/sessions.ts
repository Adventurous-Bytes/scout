import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import {
  ISession,
  IConnectivity,
  IEvent,
  ISessionWithCoordinates,
  IConnectivityWithCoordinates,
} from "../types/db";

// Input types for upsert operations
export type SessionInput = Omit<ISession, "id" | "inserted_at">;
export type SessionUpdateInput = Partial<SessionInput> & { id: number };
export type SessionUpsertInput = SessionInput | SessionUpdateInput;

export type ConnectivityInput = Omit<IConnectivity, "id" | "inserted_at">;
export type ConnectivityUpdateInput = Partial<ConnectivityInput> & {
  id: number;
};
export type ConnectivityUpsertInput =
  | ConnectivityInput
  | ConnectivityUpdateInput;

// Get sessions by herd id using RPC function with coordinates
export async function getSessionsByHerdId(
  supabase: SupabaseClient<Database>,
  herdId: number
): Promise<ISessionWithCoordinates[]> {
  const { data, error } = await supabase.rpc("get_sessions_with_coordinates", {
    herd_id_caller: herdId,
  });

  if (error) {
    throw new Error(`Failed to get sessions by herd id: ${error.message}`);
  }

  // Sort by timestamp_start in descending order
  const sortedSessions = (data || []).sort((a, b) => {
    if (!a.timestamp_start || !b.timestamp_start) return 0;
    return (
      new Date(b.timestamp_start).getTime() -
      new Date(a.timestamp_start).getTime()
    );
  });

  return sortedSessions;
}

// Get connectivity by session id using RPC function with coordinates
export async function getConnectivityBySessionId(
  supabase: SupabaseClient<Database>,
  sessionId: number
): Promise<IConnectivityWithCoordinates[]> {
  const { data, error } = await supabase.rpc(
    "get_connectivity_with_coordinates",
    { session_id_caller: sessionId }
  );

  if (error) {
    throw new Error(
      `Failed to get connectivity by session id: ${error.message}`
    );
  }

  // Sort by timestamp_start in ascending order
  const sortedConnectivity = (data || []).sort((a, b) => {
    if (!a.timestamp_start || !b.timestamp_start) return 0;
    return (
      new Date(a.timestamp_start).getTime() -
      new Date(b.timestamp_start).getTime()
    );
  });

  return sortedConnectivity;
}

// Get events by session id
export async function getEventsBySessionId(
  supabase: SupabaseClient<Database>,
  sessionId: number
): Promise<IEvent[]> {
  const { data, error } = await supabase
    .from("events")
    .select("*")
    .eq("session_id", sessionId)
    .order("timestamp_observation", { ascending: true });

  if (error) {
    throw new Error(`Failed to get events by session id: ${error.message}`);
  }

  return data || [];
}

// Create or update session
export async function upsertSession(
  supabase: SupabaseClient<Database>,
  sessionData: SessionUpsertInput
): Promise<ISession> {
  const isUpdate = "id" in sessionData;

  if (isUpdate) {
    // Update existing session
    const { id, ...updateData } = sessionData;
    const { data, error } = await supabase
      .from("sessions")
      .update(updateData)
      .eq("id", id)
      .select()
      .single();

    if (error) {
      throw new Error(`Failed to update session: ${error.message}`);
    }

    return data;
  } else {
    // Create new session
    const { data, error } = await supabase
      .from("sessions")
      .insert(sessionData)
      .select()
      .single();

    if (error) {
      throw new Error(`Failed to create session: ${error.message}`);
    }

    return data;
  }
}

// Batch upsert sessions
export async function upsertSessions(
  supabase: SupabaseClient<Database>,
  sessionsData: SessionUpsertInput[]
): Promise<ISession[]> {
  if (sessionsData.length === 0) {
    return [];
  }

  // Separate updates and inserts
  const updates = sessionsData.filter((s) => "id" in s) as SessionUpdateInput[];
  const inserts = sessionsData.filter((s) => !("id" in s)) as SessionInput[];

  const results: ISession[] = [];

  // Handle updates
  if (updates.length > 0) {
    for (const sessionData of updates) {
      try {
        const result = await upsertSession(supabase, sessionData);
        results.push(result);
      } catch (error) {
        throw new Error(`Failed to update session ${sessionData.id}: ${error}`);
      }
    }
  }

  // Handle inserts
  if (inserts.length > 0) {
    const { data, error } = await supabase
      .from("sessions")
      .insert(inserts)
      .select();

    if (error) {
      throw new Error(`Failed to create sessions: ${error.message}`);
    }

    results.push(...(data || []));
  }

  return results;
}

// Create or update connectivity
export async function upsertConnectivity(
  supabase: SupabaseClient<Database>,
  connectivityData: ConnectivityUpsertInput
): Promise<IConnectivity> {
  const isUpdate = "id" in connectivityData;

  if (isUpdate) {
    // Update existing connectivity
    const { id, ...updateData } = connectivityData;
    const { data, error } = await supabase
      .from("connectivity")
      .update(updateData)
      .eq("id", id)
      .select()
      .single();

    if (error) {
      throw new Error(`Failed to update connectivity: ${error.message}`);
    }

    return data;
  } else {
    // Create new connectivity
    const { data, error } = await supabase
      .from("connectivity")
      .insert(connectivityData)
      .select()
      .single();

    if (error) {
      throw new Error(`Failed to create connectivity: ${error.message}`);
    }

    return data;
  }
}

// Batch upsert connectivity
export async function upsertConnectivityBatch(
  supabase: SupabaseClient<Database>,
  connectivityDataArray: ConnectivityUpsertInput[]
): Promise<IConnectivity[]> {
  if (connectivityDataArray.length === 0) {
    return [];
  }

  // Separate updates and inserts
  const updates = connectivityDataArray.filter(
    (c) => "id" in c
  ) as ConnectivityUpdateInput[];
  const inserts = connectivityDataArray.filter(
    (c) => !("id" in c)
  ) as ConnectivityInput[];

  const results: IConnectivity[] = [];

  // Handle updates
  if (updates.length > 0) {
    for (const connectivityData of updates) {
      try {
        const result = await upsertConnectivity(supabase, connectivityData);
        results.push(result);
      } catch (error) {
        throw new Error(
          `Failed to update connectivity ${connectivityData.id}: ${error}`
        );
      }
    }
  }

  // Handle inserts
  if (inserts.length > 0) {
    const { data, error } = await supabase
      .from("connectivity")
      .insert(inserts)
      .select();

    if (error) {
      throw new Error(
        `Failed to create connectivity entries: ${error.message}`
      );
    }

    results.push(...(data || []));
  }

  return results;
}

// Get session with connectivity and events using RPC functions
export async function getSessionWithConnectivityAndEvents(
  supabase: SupabaseClient<Database>,
  sessionId: number
): Promise<{
  session: ISessionWithCoordinates | null;
  connectivity: IConnectivityWithCoordinates[];
  events: IEvent[];
}> {
  // Get the session from the sessions table first to get the device_id
  const { data: sessionData, error: sessionError } = await supabase
    .from("sessions")
    .select("*")
    .eq("id", sessionId)
    .single();

  if (sessionError) {
    throw new Error(`Failed to get session: ${sessionError.message}`);
  }

  // Get the device to find its herd_id
  const { data: device, error: deviceError } = await supabase
    .from("devices")
    .select("herd_id")
    .eq("id", sessionData.device_id)
    .single();

  if (deviceError) {
    throw new Error(`Failed to get device: ${deviceError.message}`);
  }

  // Get sessions with coordinates for the herd and find our specific session
  const { data: allSessionsWithCoords, error: sessionsError } =
    await supabase.rpc("get_sessions_with_coordinates", {
      herd_id_caller: device.herd_id,
    });

  if (sessionsError) {
    throw new Error(
      `Failed to get session with coordinates: ${sessionsError.message}`
    );
  }

  // Find the specific session in the results
  const sessionWithCoords =
    allSessionsWithCoords?.find((s) => s.id === sessionId) || null;

  const [connectivityResult, eventsResult] = await Promise.all([
    getConnectivityBySessionId(supabase, sessionId),
    getEventsBySessionId(supabase, sessionId),
  ]);

  return {
    session: sessionWithCoords,
    connectivity: connectivityResult,
    events: eventsResult,
  };
}

// Get sessions for a device using RPC function
export async function getSessionsByDeviceId(
  supabase: SupabaseClient<Database>,
  deviceId: number
): Promise<ISessionWithCoordinates[]> {
  const { data, error } = await supabase.rpc(
    "get_sessions_with_coordinates_by_device",
    {
      device_id_caller: deviceId,
    }
  );

  if (error) {
    throw new Error(`Failed to get sessions by device id: ${error.message}`);
  }

  return data || [];
}

// Delete session and all related data
export async function deleteSession(
  supabase: SupabaseClient<Database>,
  sessionId: number
): Promise<void> {
  const { error } = await supabase
    .from("sessions")
    .delete()
    .eq("id", sessionId);

  if (error) {
    throw new Error(`Failed to delete session: ${error.message}`);
  }
}

// Batch delete sessions
export async function deleteSessions(
  supabase: SupabaseClient<Database>,
  sessionIds: number[]
): Promise<void> {
  if (sessionIds.length === 0) {
    return;
  }

  const { error } = await supabase
    .from("sessions")
    .delete()
    .in("id", sessionIds);

  if (error) {
    throw new Error(`Failed to delete sessions: ${error.message}`);
  }
}

// Delete connectivity entry
export async function deleteConnectivity(
  supabase: SupabaseClient<Database>,
  connectivityId: number
): Promise<void> {
  const { error } = await supabase
    .from("connectivity")
    .delete()
    .eq("id", connectivityId);

  if (error) {
    throw new Error(`Failed to delete connectivity: ${error.message}`);
  }
}

// Batch delete connectivity entries
export async function deleteConnectivityBatch(
  supabase: SupabaseClient<Database>,
  connectivityIds: number[]
): Promise<void> {
  if (connectivityIds.length === 0) {
    return;
  }

  const { error } = await supabase
    .from("connectivity")
    .delete()
    .in("id", connectivityIds);

  if (error) {
    throw new Error(`Failed to delete connectivity entries: ${error.message}`);
  }
}
