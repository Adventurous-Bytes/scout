import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import {
  ISession,
  IConnectivity,
  IEvent,
  ISessionWithCoordinates,
  IConnectivityWithCoordinates,
  IEventAndTagsPrettyLocation,
  ScoutDatabaseClient,
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
  supabase: SupabaseClient<Database, "public">,
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
  supabase: SupabaseClient<Database, "public">,
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
  supabase: SupabaseClient<Database, "public">,
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

// Get events with tags by session id using RPC function
export async function getEventsAndTagsBySessionId(
  supabase: SupabaseClient<Database, "public">,
  sessionId: number,
  limit: number = 50,
  offset: number = 0
): Promise<IEventAndTagsPrettyLocation[]> {
  const { data, error } = await supabase.rpc(
    "get_events_and_tags_for_session",
    {
      session_id_caller: sessionId,
      limit_caller: limit,
      offset_caller: offset,
    }
  );

  if (error) {
    throw new Error(
      `Failed to get events and tags by session id: ${error.message}`
    );
  }

  return data || [];
}

// Get total count of events for a session
export async function getTotalEventsForSession(
  supabase: SupabaseClient<Database, "public">,
  sessionId: number
): Promise<number> {
  const { data, error } = await supabase.rpc("get_total_events_for_session", {
    session_id_caller: sessionId,
  });

  if (error) {
    throw new Error(`Failed to get total events for session: ${error.message}`);
  }

  return data || 0;
}

// Create or update session
export async function upsertSession(
  supabase: SupabaseClient<Database, "public">,
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
  supabase: SupabaseClient<Database, "public">,
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
  supabase: SupabaseClient<Database, "public">,
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
  supabase: SupabaseClient<Database, "public">,
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
  supabase: ScoutDatabaseClient,
  sessionId: number,
  herdId?: number
): Promise<{
  session: ISessionWithCoordinates | null;
  connectivity: IConnectivityWithCoordinates[];
  events: IEvent[];
}> {
  let sessionWithCoords: ISessionWithCoordinates | null = null;

  if (herdId) {
    // Use provided herd ID directly
    // Get sessions with coordinates for the herd and find our specific session
    const { data: allSessionsWithCoords, error: sessionsError } =
      await supabase.rpc("get_sessions_with_coordinates", {
        herd_id_caller: herdId,
      });

    if (sessionsError) {
      throw new Error(
        `Failed to get session with coordinates: ${sessionsError.message}`
      );
    }

    // Find the specific session in the results
    sessionWithCoords =
      allSessionsWithCoords?.find((s) => s.id === sessionId) || null;
  } else {
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
    sessionWithCoords =
      allSessionsWithCoords?.find((s) => s.id === sessionId) || null;
  }

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

// Get session with connectivity and events with tags using RPC functions
export async function getSessionWithConnectivityAndEventsWithTags(
  supabase: ScoutDatabaseClient,
  sessionId: number,
  limit: number = 50,
  offset: number = 0,
  herdId?: number
): Promise<{
  session: ISessionWithCoordinates | null;
  connectivity: IConnectivityWithCoordinates[];
  eventsWithTags: IEventAndTagsPrettyLocation[];
  totalEvents: number;
}> {
  let sessionWithCoords: ISessionWithCoordinates | null = null;
  let actualHerdId: number;

  if (herdId) {
    // Use provided herd ID directly
    actualHerdId = herdId;

    // Get sessions with coordinates for the herd and find our specific session
    const { data: allSessionsWithCoords, error: sessionsError } =
      await supabase.rpc("get_sessions_with_coordinates", {
        herd_id_caller: actualHerdId,
      });

    if (sessionsError) {
      throw new Error(
        `Failed to get session with coordinates: ${sessionsError.message}`
      );
    }

    // Find the specific session in the results
    sessionWithCoords =
      allSessionsWithCoords?.find((s) => s.id === sessionId) || null;
  } else {
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

    actualHerdId = device.herd_id;

    // Get sessions with coordinates for the herd and find our specific session
    const { data: allSessionsWithCoords, error: sessionsError } =
      await supabase.rpc("get_sessions_with_coordinates", {
        herd_id_caller: actualHerdId,
      });

    if (sessionsError) {
      throw new Error(
        `Failed to get session with coordinates: ${sessionsError.message}`
      );
    }

    // Find the specific session in the results
    sessionWithCoords =
      allSessionsWithCoords?.find((s) => s.id === sessionId) || null;
  }

  const [connectivityResult, eventsWithTagsResult, totalEventsResult] =
    await Promise.all([
      getConnectivityBySessionId(supabase, sessionId),
      getEventsAndTagsBySessionId(supabase, sessionId, limit, offset),
      getTotalEventsForSession(supabase, sessionId),
    ]);

  return {
    session: sessionWithCoords,
    connectivity: connectivityResult,
    eventsWithTags: eventsWithTagsResult,
    totalEvents: totalEventsResult,
  };
}

// Get sessions for a device using RPC function
export async function getSessionsByDeviceId(
  supabase: ScoutDatabaseClient,
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
  supabase: ScoutDatabaseClient,
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
  supabase: ScoutDatabaseClient,
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
  supabase: ScoutDatabaseClient,
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
  supabase: ScoutDatabaseClient,
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
