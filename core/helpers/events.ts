"use server";

import { newServerClient } from "../supabase/server";
import { IEvent } from "../types/db";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { addSignedUrlsToEvents } from "./storage";

export async function server_get_events_by_herd(
  herd_id: number
): Promise<IWebResponseCompatible<IEvent[]>> {
  const supabase = await newServerClient();
  // fetch events and include devices
  // sort by timestamp
  const { data }: { data: IEvent[] | null } = await supabase
    .from("events")
    .select(
      `
      *,
      devices: devices!inner(*)
      `
    )
    .eq("devices.herd_id", herd_id)
    .order("timestamp_observation", { ascending: false });

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
  page_count: number = 10
): Promise<IWebResponseCompatible<IEvent[]>> {
  const from = offset * page_count;
  const to = from + page_count - 1;

  const supabase = await newServerClient();
  // fetch events and include devices
  // sort by timestamp
  const { data }: { data: IEvent[] | null } = await supabase
    .from("events")
    .select(
      `
      *,
      devices: devices!inner(*)
      `
    )
    .eq("devices.herd_id", herd_id)
    .range(from, to)
    .order("timestamp_observation", { ascending: false });

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
  herd_id: number
): Promise<IWebResponseCompatible<number>> {
  const supabase = await newServerClient();
  // call public.get_total_events_for_herd(herd_id)
  const { data, error } = await supabase.rpc("get_total_events_for_herd", {
    herd_id_caller: herd_id,
  });
  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: 0,
    };
  } else {
    return IWebResponse.success(data).to_compatible();
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
