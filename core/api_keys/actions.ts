"use server";

import { newServerClient } from "../supabase/server";
import { IApiKeyScout } from "../types/db";

// Test function to verify individual API key loading works
export async function test_api_key_loading(
  device_id: number
): Promise<boolean> {
  try {
    console.log(
      `[API Key Test] Testing individual API key loading for device ${device_id}`
    );
    const api_keys = await server_list_api_keys(device_id);
    console.log(
      `[API Key Test] Successfully loaded ${api_keys.length} API keys for device ${device_id}`
    );
    return true;
  } catch (error) {
    console.error(
      `[API Key Test] Failed to load API keys for device ${device_id}:`,
      error
    );
    return false;
  }
}

export async function server_list_api_keys(
  device_id: number
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
    // Parse the JSON string from the text array
    const converted_data: IApiKeyScout = JSON.parse(data[i]);
    data_to_return.push(converted_data);
  }
  return data_to_return;
}

export async function server_list_api_keys_batch(
  device_ids: number[]
): Promise<{ [device_id: number]: IApiKeyScout[] }> {
  const supabase = await newServerClient();

  // Check if the batch function exists by trying a simple call
  try {
    const { data, error } = await supabase.rpc("load_api_keys_batch", {
      device_ids: device_ids,
    });

    if (error) {
      // Check if it's a "function does not exist" error
      if (
        error.message.includes("function") &&
        error.message.includes("does not exist")
      ) {
        console.log(
          `[API Keys Batch] Batch function not deployed, using individual calls...`
        );
      } else {
        console.error(`[API Keys Batch] Database error:`, error.message);
      }
      console.log(`[API Keys Batch] Falling back to individual calls...`);

      // Fallback to individual API key loading
      const result: { [device_id: number]: IApiKeyScout[] } = {};
      const promises = device_ids.map(async (device_id) => {
        try {
          const api_keys = await server_list_api_keys(device_id);
          result[device_id] = api_keys;
        } catch (err) {
          console.warn(`[API Keys Batch] Failed for device ${device_id}:`, err);
          result[device_id] = [];
        }
      });

      await Promise.all(promises);
      return result;
    }

    if (!data) {
      return {};
    }

    const result: { [device_id: number]: IApiKeyScout[] } = {};

    // Group API keys by device_id
    data.forEach((item) => {
      const device_id = item.device_id;
      if (!result[device_id]) {
        result[device_id] = [];
      }

      result[device_id].push({
        id: item.api_key_id, // Now a string, no need for toString()
        key: item.api_key_key,
      });
    });

    return result;
  } catch (err) {
    console.error(`[API Keys Batch] Unexpected error:`, err);
    console.log(`[API Keys Batch] Falling back to individual calls...`);

    // Fallback to individual API key loading
    const result: { [device_id: number]: IApiKeyScout[] } = {};
    const promises = device_ids.map(async (device_id) => {
      try {
        const api_keys = await server_list_api_keys(device_id);
        result[device_id] = api_keys;
      } catch (err) {
        console.warn(`[API Keys Batch] Failed for device ${device_id}:`, err);
        result[device_id] = [];
      }
    });

    await Promise.all(promises);
    return result;
  }
}
