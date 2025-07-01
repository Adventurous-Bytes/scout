"use server";

import { newServerClient } from "../supabase/server";
import { IApiKeyScout } from "../types/db";

export async function server_list_api_keys(
  device_id: string
): Promise<IApiKeyScout[]> {
  const supabase = await newServerClient();
  const { data, error } = await supabase.rpc("load_api_keys", {
    id_of_device: device_id,
  });
  if (error) {
    console.error("Error listing API keys:", error.message);
  }
  if (!data) return [];
  const data_to_return: IApiKeyScout[] = [];
  for (let i = 0; i < data.length; i++) {
    // convert data to IApiKeyScout
    const converted_data: IApiKeyScout = JSON.parse(data[i]);
    data_to_return.push(converted_data);
  }
  return data_to_return;
}
