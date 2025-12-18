"use server";

import { newServerClient } from "../supabase/server";
import { ISessionSummary } from "../types/db";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";

export async function server_get_session_summaries_by_herd(
  herd_id: number,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<ISessionSummary>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase.rpc("get_session_summaries", {
    start_date_caller: undefined,
    end_date_caller: undefined,
    device_id_caller: undefined,
    herd_id_caller: herd_id,
  });

  if (error) {
    return IWebResponse.error<ISessionSummary>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<ISessionSummary>(
      "No session summary data returned",
    ).to_compatible();
  }

  return IWebResponse.success(
    data as unknown as ISessionSummary,
  ).to_compatible();
}

export async function server_get_session_summaries_by_device(
  device_id: number,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<ISessionSummary>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase.rpc("get_session_summaries", {
    start_date_caller: undefined,
    end_date_caller: undefined,
    device_id_caller: device_id,
    herd_id_caller: undefined,
  });

  if (error) {
    return IWebResponse.error<ISessionSummary>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<ISessionSummary>(
      "No session summary data returned",
    ).to_compatible();
  }

  return IWebResponse.success(
    data as unknown as ISessionSummary,
  ).to_compatible();
}

export async function server_get_session_summaries_with_filters(
  herd_id?: number,
  device_id?: number,
  start_date?: string,
  end_date?: string,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<ISessionSummary>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase.rpc("get_session_summaries", {
    start_date_caller: start_date || undefined,
    end_date_caller: end_date || undefined,
    device_id_caller: device_id || undefined,
    herd_id_caller: herd_id || undefined,
  });

  if (error) {
    return IWebResponse.error<ISessionSummary>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<ISessionSummary>(
      "No session summary data returned",
    ).to_compatible();
  }

  return IWebResponse.success(
    data as unknown as ISessionSummary,
  ).to_compatible();
}
