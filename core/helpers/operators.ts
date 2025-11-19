"use server";

import { newServerClient } from "../supabase/server";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { IOperator } from "../types/db";

// Get operators by session id (server id)
export async function server_get_operators_by_session_id(
  sessionId: number,
): Promise<IWebResponseCompatible<IOperator[]>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase
    .from("operators")
    .select("*")
    .eq("session_id", sessionId)
    .order("created_at", { ascending: false });

  if (error) {
    console.warn("Error fetching operators by session id:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  return IWebResponse.success(data || []).to_compatible();
}

// Get all operators for a specific user
export async function server_get_operators_by_user_id(
  userId: string,
): Promise<IWebResponseCompatible<IOperator[]>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase
    .from("operators")
    .select("*")
    .eq("user_id", userId)
    .order("created_at", { ascending: false });

  if (error) {
    console.warn("Error fetching operators by user id:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  return IWebResponse.success(data || []).to_compatible();
}

// Get operators by session id with additional filters
export async function server_get_operators_by_session_id_filtered(
  sessionId: number,
  action?: string,
  timestampAfter?: string,
): Promise<IWebResponseCompatible<IOperator[]>> {
  const supabase = await newServerClient();

  let query = supabase
    .from("operators")
    .select("*")
    .eq("session_id", sessionId);

  // Apply optional filters
  if (action) {
    query = query.eq("action", action);
  }

  if (timestampAfter) {
    query = query.gte("timestamp", timestampAfter);
  }

  const { data, error } = await query.order("timestamp", { ascending: false });

  if (error) {
    console.warn(
      "Error fetching filtered operators by session id:",
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
