"use server";

import { newServerClient } from "../supabase/server";
import {
  ISessionWithCoordinates,
  ISessionUsageOverTime,
  IEventAndTagsPrettyLocation,
  ISession,
  SessionInsert,
  SessionUpdate,
} from "../types/db";
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

// Insert new sessions (accepts array for batch operations)
export async function server_insert_session(
  sessions: SessionInsert | SessionInsert[],
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<ISession[]>> {
  const supabase = client || (await newServerClient());

  const sessionsArray = Array.isArray(sessions) ? sessions : [sessions];

  const { data, error } = await supabase
    .from("sessions")
    .insert(sessionsArray)
    .select("*");

  if (error) {
    console.warn("Error inserting sessions:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  return IWebResponse.success(data || []).to_compatible();
}

// Update existing sessions (accepts array for batch operations)
// Each session in the array must include an 'id' field
export async function server_update_session(
  sessions: (SessionUpdate & { id: number }) | (SessionUpdate & { id: number })[],
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<ISession[]>> {
  const supabase = client || (await newServerClient());

  const sessionsArray = Array.isArray(sessions) ? sessions : [sessions];
  const updatedSessions: ISession[] = [];

  for (const session of sessionsArray) {
    const { id, ...updateData } = session;
    // Remove fields that shouldn't be updated
    delete (updateData as any).inserted_at;

    const { data, error } = await supabase
      .from("sessions")
      .update(updateData)
      .eq("id", id)
      .select("*")
      .single();

    if (error) {
      console.warn("Error updating session:", error.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: [],
      };
    }

    if (data) {
      updatedSessions.push(data);
    }
  }

  return IWebResponse.success(updatedSessions).to_compatible();
}
