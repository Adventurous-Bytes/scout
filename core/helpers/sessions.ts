"use server";

import { newServerClient } from "../supabase/server";
import {
  ISession,
  IConnectivity,
  IEvent,
  ISessionWithCoordinates,
  IConnectivityWithCoordinates,
  IEventAndTagsPrettyLocation,
} from "../types/db";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { server_get_connectivity_by_session_id } from "./connectivity";

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
export async function server_get_sessions_by_herd_id(
  herdId: number,
): Promise<IWebResponseCompatible<ISessionWithCoordinates[]>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase.rpc("get_sessions_with_coordinates", {
    herd_id_caller: herdId,
  });

  if (error) {
    console.warn("Error fetching sessions by herd id:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  // Sort by timestamp_start in descending order
  const sortedSessions = (data || []).sort((a, b) => {
    if (!a.timestamp_start || !b.timestamp_start) return 0;
    return (
      new Date(b.timestamp_start).getTime() -
      new Date(a.timestamp_start).getTime()
    );
  });
  return IWebResponse.success(sortedSessions).to_compatible();
}

// Get events by session id
export async function server_get_events_by_session_id(
  sessionId: number,
): Promise<IWebResponseCompatible<IEvent[]>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase
    .from("events")
    .select("*")
    .eq("session_id", sessionId)
    .order("timestamp_observation", { ascending: true });

  if (error) {
    console.warn("Error fetching events by session id:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  return IWebResponse.success(data || []).to_compatible();
}

// Get events with tags by session id using RPC function
export async function server_get_events_and_tags_by_session_id(
  sessionId: number,
  limit: number = 50,
  offset: number = 0,
): Promise<IWebResponseCompatible<IEventAndTagsPrettyLocation[]>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase.rpc(
    "get_events_and_tags_for_session",
    {
      session_id_caller: sessionId,
      limit_caller: limit,
      offset_caller: offset,
    },
  );

  if (error) {
    console.warn(
      "Error fetching events and tags by session id:",
      error.message,
    );
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  return IWebResponse.success(data || []).to_compatible();
}

// Get total count of events for a session
export async function server_get_total_events_for_session(
  sessionId: number,
): Promise<IWebResponseCompatible<number>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase.rpc("get_total_events_for_session", {
    session_id_caller: sessionId,
  });

  if (error) {
    console.warn("Error fetching total events for session:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: 0,
    };
  }

  return IWebResponse.success(data || 0).to_compatible();
}

// Create or update session
export async function server_upsert_session(
  sessionData: SessionUpsertInput,
): Promise<IWebResponseCompatible<ISession>> {
  const supabase = await newServerClient();
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
      console.warn("Error updating session:", error.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: null,
      };
    }

    return IWebResponse.success(data).to_compatible();
  } else {
    // Create new session
    const { data, error } = await supabase
      .from("sessions")
      .insert(sessionData)
      .select()
      .single();

    if (error) {
      console.warn("Error creating session:", error.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: null,
      };
    }

    return IWebResponse.success(data).to_compatible();
  }
}

// Batch upsert sessions
export async function server_upsert_sessions(
  sessionsData: SessionUpsertInput[],
): Promise<IWebResponseCompatible<ISession[]>> {
  const supabase = await newServerClient();

  if (sessionsData.length === 0) {
    return IWebResponse.success([]).to_compatible();
  }

  // Separate updates and inserts
  const updates = sessionsData.filter((s) => "id" in s) as SessionUpdateInput[];
  const inserts = sessionsData.filter((s) => !("id" in s)) as SessionInput[];

  const results: ISession[] = [];

  // Handle updates
  if (updates.length > 0) {
    for (const sessionData of updates) {
      const updateResult = await server_upsert_session(sessionData);
      if (
        updateResult.status === EnumWebResponse.SUCCESS &&
        updateResult.data
      ) {
        results.push(updateResult.data);
      } else {
        console.warn(
          `Failed to update session ${sessionData.id}:`,
          updateResult.msg,
        );
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
      console.warn("Error creating sessions:", error.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: results,
      };
    }

    results.push(...(data || []));
  }

  return IWebResponse.success(results).to_compatible();
}

// Create or update connectivity
export async function server_upsert_connectivity(
  connectivityData: ConnectivityUpsertInput,
): Promise<IWebResponseCompatible<IConnectivity>> {
  const supabase = await newServerClient();
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
      console.warn("Error updating connectivity:", error.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: null,
      };
    }

    return IWebResponse.success(data).to_compatible();
  } else {
    // Create new connectivity
    const { data, error } = await supabase
      .from("connectivity")
      .insert(connectivityData)
      .select()
      .single();

    if (error) {
      console.warn("Error creating connectivity:", error.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: null,
      };
    }

    return IWebResponse.success(data).to_compatible();
  }
}

// Batch upsert connectivity
export async function server_upsert_connectivity_batch(
  connectivityDataArray: ConnectivityUpsertInput[],
): Promise<IWebResponseCompatible<IConnectivity[]>> {
  const supabase = await newServerClient();

  if (connectivityDataArray.length === 0) {
    return IWebResponse.success([]).to_compatible();
  }

  // Separate updates and inserts
  const updates = connectivityDataArray.filter(
    (c) => "id" in c,
  ) as ConnectivityUpdateInput[];
  const inserts = connectivityDataArray.filter(
    (c) => !("id" in c),
  ) as ConnectivityInput[];

  const results: IConnectivity[] = [];

  // Handle updates
  if (updates.length > 0) {
    for (const connectivityData of updates) {
      const updateResult = await server_upsert_connectivity(connectivityData);
      if (
        updateResult.status === EnumWebResponse.SUCCESS &&
        updateResult.data
      ) {
        results.push(updateResult.data);
      } else {
        console.warn(
          `Failed to update connectivity ${connectivityData.id}:`,
          updateResult.msg,
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
      console.warn("Error creating connectivity entries:", error.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: results,
      };
    }

    results.push(...(data || []));
  }

  return IWebResponse.success(results).to_compatible();
}

// Get session with connectivity and events using RPC functions
export async function server_get_session_with_connectivity_and_events(
  sessionId: number,
  herdId?: number,
): Promise<
  IWebResponseCompatible<{
    session: ISessionWithCoordinates | null;
    connectivity: IConnectivityWithCoordinates[];
    events: IEvent[];
  }>
> {
  const supabase = await newServerClient();
  let sessionWithCoords: ISessionWithCoordinates | null = null;

  if (herdId) {
    // Use provided herd ID directly
    const sessionsResult = await server_get_sessions_by_herd_id(herdId);
    if (
      sessionsResult.status === EnumWebResponse.SUCCESS &&
      sessionsResult.data
    ) {
      sessionWithCoords =
        sessionsResult.data.find((s) => s.id === sessionId) || null;
    }
  } else {
    // Get the session from the sessions table first to get the device_id
    const { data: sessionData, error: sessionError } = await supabase
      .from("sessions")
      .select("*")
      .eq("id", sessionId)
      .single();

    if (sessionError) {
      console.warn("Error getting session:", sessionError.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: sessionError.message,
        data: {
          session: null,
          connectivity: [],
          events: [],
        },
      };
    }

    // Get the device to find its herd_id
    const { data: device, error: deviceError } = await supabase
      .from("devices")
      .select("herd_id")
      .eq("id", sessionData.device_id)
      .single();

    if (deviceError) {
      console.warn("Error getting device:", deviceError.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: deviceError.message,
        data: {
          session: null,
          connectivity: [],
          events: [],
        },
      };
    }

    const sessionsResult = await server_get_sessions_by_herd_id(device.herd_id);
    if (
      sessionsResult.status === EnumWebResponse.SUCCESS &&
      sessionsResult.data
    ) {
      sessionWithCoords =
        sessionsResult.data.find((s) => s.id === sessionId) || null;
    }
  }

  const [connectivityResult, eventsResult] = await Promise.all([
    server_get_connectivity_by_session_id(sessionId),
    server_get_events_by_session_id(sessionId),
  ]);

  const connectivity =
    connectivityResult.status === EnumWebResponse.SUCCESS
      ? connectivityResult.data || []
      : [];
  const events =
    eventsResult.status === EnumWebResponse.SUCCESS
      ? eventsResult.data || []
      : [];

  return IWebResponse.success({
    session: sessionWithCoords,
    connectivity,
    events,
  }).to_compatible();
}

// Get session with connectivity and events with tags using RPC functions
export async function server_get_session_with_connectivity_and_events_with_tags(
  sessionId: number,
  limit: number = 50,
  offset: number = 0,
  herdId?: number,
): Promise<
  IWebResponseCompatible<{
    session: ISessionWithCoordinates | null;
    connectivity: IConnectivityWithCoordinates[];
    eventsWithTags: IEventAndTagsPrettyLocation[];
    totalEvents: number;
  }>
> {
  const supabase = await newServerClient();
  let sessionWithCoords: ISessionWithCoordinates | null = null;
  let actualHerdId: number;

  if (herdId) {
    // Use provided herd ID directly
    actualHerdId = herdId;
    const sessionsResult = await server_get_sessions_by_herd_id(herdId);
    if (
      sessionsResult.status === EnumWebResponse.SUCCESS &&
      sessionsResult.data
    ) {
      sessionWithCoords =
        sessionsResult.data.find((s) => s.id === sessionId) || null;
    }
  } else {
    // Get the session from the sessions table first to get the device_id
    const { data: sessionData, error: sessionError } = await supabase
      .from("sessions")
      .select("*")
      .eq("id", sessionId)
      .single();

    if (sessionError) {
      console.warn("Error getting session:", sessionError.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: sessionError.message,
        data: {
          session: null,
          connectivity: [],
          eventsWithTags: [],
          totalEvents: 0,
        },
      };
    }

    // Get the device to find its herd_id
    const { data: device, error: deviceError } = await supabase
      .from("devices")
      .select("herd_id")
      .eq("id", sessionData.device_id)
      .single();

    if (deviceError) {
      console.warn("Error getting device:", deviceError.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: deviceError.message,
        data: {
          session: null,
          connectivity: [],
          eventsWithTags: [],
          totalEvents: 0,
        },
      };
    }

    actualHerdId = device.herd_id;
    const sessionsResult = await server_get_sessions_by_herd_id(device.herd_id);
    if (
      sessionsResult.status === EnumWebResponse.SUCCESS &&
      sessionsResult.data
    ) {
      sessionWithCoords =
        sessionsResult.data.find((s) => s.id === sessionId) || null;
    }
  }

  const [connectivityResult, eventsWithTagsResult, totalEventsResult] =
    await Promise.all([
      server_get_connectivity_by_session_id(sessionId),
      server_get_events_and_tags_by_session_id(sessionId, limit, offset),
      server_get_total_events_for_session(sessionId),
    ]);

  const connectivity =
    connectivityResult.status === EnumWebResponse.SUCCESS
      ? connectivityResult.data || []
      : [];
  const eventsWithTags =
    eventsWithTagsResult.status === EnumWebResponse.SUCCESS
      ? eventsWithTagsResult.data || []
      : [];
  const totalEvents =
    totalEventsResult.status === EnumWebResponse.SUCCESS
      ? totalEventsResult.data || 0
      : 0;

  return IWebResponse.success({
    session: sessionWithCoords,
    connectivity,
    eventsWithTags,
    totalEvents,
  }).to_compatible();
}

// Get sessions for a device using RPC function
export async function server_get_sessions_by_device_id(
  deviceId: number,
): Promise<IWebResponseCompatible<ISessionWithCoordinates[]>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase.rpc(
    "get_sessions_with_coordinates_by_device",
    {
      device_id_caller: deviceId,
    },
  );

  if (error) {
    console.warn("Error fetching sessions by device id:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  return IWebResponse.success(data || []).to_compatible();
}

// Delete session and all related data
export async function server_delete_session(
  sessionId: number,
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();

  const { error } = await supabase
    .from("sessions")
    .delete()
    .eq("id", sessionId);

  if (error) {
    console.warn("Error deleting session:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: false,
    };
  }

  return IWebResponse.success(true).to_compatible();
}

// Batch delete sessions
export async function server_delete_sessions(
  sessionIds: number[],
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();

  if (sessionIds.length === 0) {
    return IWebResponse.success(true).to_compatible();
  }

  const { error } = await supabase
    .from("sessions")
    .delete()
    .in("id", sessionIds);

  if (error) {
    console.warn("Error deleting sessions:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: false,
    };
  }

  return IWebResponse.success(true).to_compatible();
}

// Delete connectivity entry
export async function server_delete_connectivity(
  connectivityId: number,
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();

  const { error } = await supabase
    .from("connectivity")
    .delete()
    .eq("id", connectivityId);

  if (error) {
    console.warn("Error deleting connectivity:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: false,
    };
  }

  return IWebResponse.success(true).to_compatible();
}

// Batch delete connectivity entries
export async function server_delete_connectivity_batch(
  connectivityIds: number[],
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();

  if (connectivityIds.length === 0) {
    return IWebResponse.success(true).to_compatible();
  }

  const { error } = await supabase
    .from("connectivity")
    .delete()
    .in("id", connectivityIds);

  if (error) {
    console.warn("Error deleting connectivity entries:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: false,
    };
  }

  return IWebResponse.success(true).to_compatible();
}
