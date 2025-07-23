"use server";

import { newServerClient } from "../supabase/server";
import { IEvent } from "../types/db";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { addSignedUrlsToEvents } from "./storage";

export enum EnumSessionsVisibility {
  Only = 0,
  Exclude = 1,
  Combine = 2,
}

export async function server_get_events_by_herd(
  herd_id: number,
  sessions_visibility: EnumSessionsVisibility
): Promise<IWebResponseCompatible<IEvent[]>> {
  const supabase = await newServerClient();
  // fetch events and include devices
  // sort by timestamp
  let query = supabase
    .from("events")
    .select(
      `
      *,
      devices: devices!inner(*)
      `
    )
    .eq("devices.herd_id", herd_id);

  // Apply session filter based on sessions_visibility
  if (sessions_visibility === EnumSessionsVisibility.Only) {
    query = query.not("session_id", "is", null);
  } else if (sessions_visibility === EnumSessionsVisibility.Exclude) {
    query = query.is("session_id", null);
  }

  const { data }: { data: IEvent[] | null } = await query.order(
    "timestamp_observation",
    { ascending: false }
  );

  // Add signed URLs to events using the same client
  const eventsWithSignedUrls = data
    ? await addSignedUrlsToEvents(data, supabase)
    : [];

  // TODO: DETERMINE WHEN TO PASS ERROR
  let response: IWebResponse<IEvent[]> =
    IWebResponse.success(eventsWithSignedUrls);
  return response.to_compatible();
}

export async function server_get_more_events_by_herd(
  herd_id: number,
  offset: number,
  page_count: number = 10,
  includeSessionEvents: boolean
): Promise<IWebResponseCompatible<IEvent[]>> {
  const from = offset * page_count;
  const to = from + page_count - 1;

  const supabase = await newServerClient();
  // fetch events and include devices
  // sort by timestamp
  let query = supabase
    .from("events")
    .select(
      `
      *,
      devices: devices!inner(*)
      `
    )
    .eq("devices.herd_id", herd_id)
    .range(from, to);

  if (includeSessionEvents) {
    // Include only events that have a session_id
    query = query.not("session_id", "is", null);
  } else {
    // Include only events that don't have a session_id
    query = query.is("session_id", null);
  }

  const { data }: { data: IEvent[] | null } = await query.order(
    "timestamp_observation",
    { ascending: false }
  );

  // Add signed URLs to events using the same client
  const eventsWithSignedUrls = data
    ? await addSignedUrlsToEvents(data, supabase)
    : [];

  // TODO: DETERMINE WHEN TO PASS ERROR
  let response: IWebResponse<IEvent[]> =
    IWebResponse.success(eventsWithSignedUrls);
  return response.to_compatible();
}

// function to get total number of events for a herd
export async function server_get_total_events_by_herd(
  herd_id: number,
  sessions_visibility: EnumSessionsVisibility
): Promise<IWebResponseCompatible<number>> {
  const supabase = await newServerClient();

  let query = supabase
    .from("events")
    .select("id", { count: "exact", head: true })
    .eq("devices.herd_id", herd_id);

  if (sessions_visibility === EnumSessionsVisibility.Only) {
    query = query.not("session_id", "is", null);
  } else if (sessions_visibility === EnumSessionsVisibility.Exclude) {
    query = query.is("session_id", null);
  }

  const { count, error } = await query;

  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: 0,
    };
  } else {
    return IWebResponse.success(count || 0).to_compatible();
  }
}

export async function server_create_event(
  newEvent: any
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();

  // strip id field from herd object
  const { data, error } = await supabase.from("events").insert([newEvent]);
  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  } else {
    return IWebResponse.success(true).to_compatible();
  }
}
