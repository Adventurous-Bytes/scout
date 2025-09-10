"use server";

import { newServerClient } from "../supabase/server";
import { IWebResponseCompatible, IWebResponse, EnumWebResponse } from "../types/requests";
import { TablesInsert } from "../types/supabase";

export type ChatMessageInsert = TablesInsert<"chat">;

export async function server_insert_chat_message(
  message: string,
  herd_id: number,
  sender?: string
): Promise<IWebResponseCompatible<{ id: number }>> {
  const supabase = await newServerClient();
  const chatMessage: ChatMessageInsert = {
    message,
    herd_id,
    sender: sender || null,
  };

  const { data, error } = await supabase
    .from("chat")
    .insert(chatMessage)
    .select("id")
    .single();

  if (error) {
    console.warn("Error inserting chat message:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: `Failed to insert chat message: ${error.message}`,
      data: null,
    };
  }

  return IWebResponse.success({ id: data.id }).to_compatible();
}

export async function server_get_chat_messages(
  limit: number = 50,
  offset: number = 0,
  herd_id: number
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
    .range(offset, offset + limit - 1)
    .eq("herd_id", herd_id);

  if (error) {
    console.warn("Error fetching chat messages:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: `Failed to fetch chat messages: ${error.message}`,
      data: [],
    };
  }

  return IWebResponse.success(data || []).to_compatible();
}
