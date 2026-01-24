"use server";

import { newServerClient } from "../supabase/server";

import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";

import { EnumSessionsVisibility } from "../types/events";
import { IEvent, EventInsert, EventUpdate } from "../types/db";
import { SupabaseClient } from "@supabase/supabase-js";

// function to get total number of events for a herd
export async function server_get_total_events_by_herd(
  herd_id: number,
  sessions_visibility: EnumSessionsVisibility
): Promise<IWebResponseCompatible<number>> {
  const supabase = await newServerClient();

  // Convert sessions_visibility to exclude_session_events boolean
  const exclude_session_events =
    sessions_visibility === EnumSessionsVisibility.Exclude;

  const { data, error } = (await supabase.rpc(
    "get_total_events_for_herd_with_session_filter",
    {
      herd_id_caller: herd_id,
      exclude_session_events: exclude_session_events,
    }
  )) as { data: number | null; error: any };

  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: 0,
    };
  } else {
    return IWebResponse.success(data || 0).to_compatible();
  }
}

// Insert a new event
export async function server_insert_event(
  event: EventInsert,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IEvent | null>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase
    .from("events")
    .insert([event])
    .select("*")
    .single();

  if (error) {
    console.warn("Error inserting event:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  }

  return IWebResponse.success(data).to_compatible();
}

// Update an existing event
export async function server_update_event(
  eventId: number,
  updates: EventUpdate,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IEvent | null>> {
  const supabase = client || (await newServerClient());

  // Remove fields that shouldn't be updated
  const updateData = { ...updates };
  delete (updateData as any).id;
  delete (updateData as any).inserted_at;

  const { data, error } = await supabase
    .from("events")
    .update(updateData)
    .eq("id", eventId)
    .select("*")
    .single();

  if (error) {
    console.warn("Error updating event:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  }

  if (!data) {
    return {
      status: EnumWebResponse.ERROR,
      msg: "Event not found or update failed",
      data: null,
    };
  }

  return IWebResponse.success(data).to_compatible();
}
