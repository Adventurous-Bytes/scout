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

// Insert new events (accepts array for batch operations)
export async function server_insert_event(
  events: EventInsert | EventInsert[],
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IEvent[]>> {
  const supabase = client || (await newServerClient());

  const eventsArray = Array.isArray(events) ? events : [events];

  const { data, error } = await supabase
    .from("events")
    .insert(eventsArray)
    .select("*");

  if (error) {
    console.warn("Error inserting events:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  return IWebResponse.success(data || []).to_compatible();
}

// Update existing events (accepts array for batch operations)
// Each event in the array must include an 'id' field
export async function server_update_event(
  events: (EventUpdate & { id: number }) | (EventUpdate & { id: number })[],
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IEvent[]>> {
  const supabase = client || (await newServerClient());

  const eventsArray = Array.isArray(events) ? events : [events];
  const updatedEvents: IEvent[] = [];

  for (const event of eventsArray) {
    const { id, ...updateData } = event;
    // Remove fields that shouldn't be updated
    delete (updateData as any).inserted_at;

    const { data, error } = await supabase
      .from("events")
      .update(updateData)
      .eq("id", id)
      .select("*")
      .single();

    if (error) {
      console.warn("Error updating event:", error.message);
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: [],
      };
    }

    if (data) {
      updatedEvents.push(data);
    }
  }

  return IWebResponse.success(updatedEvents).to_compatible();
}
