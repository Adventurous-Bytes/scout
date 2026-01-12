import { createClient, SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { getSupabaseUrl, getSupabaseAnonKey } from "../constants/env";

export function createClientWithApiKey(
  user_api_key: string
): SupabaseClient<Database> | null {
  const supabase_url = getSupabaseUrl();
  const supabase_anon_key = getSupabaseAnonKey();

  if (!supabase_url || !supabase_anon_key) {
    return null;
  }
  // EXCHANGE API KEY FOR JWT
  const supabase_anon = createClient<Database>(
    supabase_url,
    supabase_anon_key,
    {
      global: {
        headers: {
          api_key: `${user_api_key}`,
        },
      },
      auth: {
        persistSession: false,
        detectSessionInUrl: false,
        autoRefreshToken: false,
      },
    }
  );
  return supabase_anon;
}
