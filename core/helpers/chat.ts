"use server";

import { newServerClient } from "../supabase/server";
import {
  IWebResponseCompatible,
  IWebResponse,
  EnumWebResponse,
} from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";
import { TablesInsert } from "../types/supabase";

export type ChatMessageInsert = TablesInsert<"chat">;

export async function server_insert_chat_message(
  message: string,
  sender?: string
): Promise<IWebResponseCompatible<{ id: number }>> {
  const supabase = await newServerClient();

  const chatMessage: ChatMessageInsert = {
    message,
    sender: sender || null,
  };

  const { data, error } = await supabase
    .from("chat")
    .insert(chatMessage)
    .select("id")
    .single();

  if (error) {
    return IWebResponse.error<{ id: number }>(
      `Failed to insert chat message: ${error.message}`
    ).to_compatible();
  }

  return IWebResponse.success({ id: data.id }).to_compatible();
}

export async function server_insert_chat_message_with_client(
  message: string,
  supabaseClient: SupabaseClient,
  sender?: string
): Promise<IWebResponseCompatible<{ id: number }>> {
  const chatMessage: ChatMessageInsert = {
    message,
    sender: sender || null,
  };

  const { data, error } = await supabaseClient
    .from("chat")
    .insert(chatMessage)
    .select("id")
    .single();

  if (error) {
    return IWebResponse.error<{ id: number }>(
      `Failed to insert chat message: ${error.message}`
    ).to_compatible();
  }

  return IWebResponse.success({ id: data.id }).to_compatible();
}

export async function server_get_chat_messages(
  limit: number = 50,
  offset: number = 0
): Promise<
  IWebResponseCompatible<
    Array<{
      id: number;
      message: string;
      sender: string | null;
      created_at: string;
    }>
  >
> {
  const supabase = await newServerClient();

  const { data, error } = await supabase
    .from("chat")
    .select("id, message, sender, created_at")
    .order("created_at", { ascending: false })
    .range(offset, offset + limit - 1);

  if (error) {
    return IWebResponse.error<
      Array<{
        id: number;
        message: string;
        sender: string | null;
        created_at: string;
      }>
    >(`Failed to fetch chat messages: ${error.message}`).to_compatible();
  }

  return IWebResponse.success(data || []).to_compatible();
}
