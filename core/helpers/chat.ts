import { IWebResponseCompatible, IWebResponse } from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";
import { TablesInsert } from "../types/supabase";

export type ChatMessageInsert = TablesInsert<"chat">;

export async function insert_chat_message(
  supabaseClient: SupabaseClient,
  message: string,
  herd_id: number,
  sender?: string
): Promise<IWebResponseCompatible<{ id: number }>> {
  const chatMessage: ChatMessageInsert = {
    message,
    herd_id,
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

export async function get_chat_messages(
  supabaseClient: SupabaseClient,
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
  const { data, error } = await supabaseClient
    .from("chat")
    .select("id, message, sender, created_at")
    .order("created_at", { ascending: false })
    .range(offset, offset + limit - 1)
    .eq("herd_id", herd_id);

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
