"use server";

import { newServerClient } from "../supabase/server";

import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";

import { EnumSessionsVisibility } from "../types/events";

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
