import { createClient, SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";

export function createClientWithApiKey(
  user_api_key: string
): SupabaseClient<Database> | null {
  // Assumes Next.js environment variables (NEXT_PUBLIC_*)
  const supabase_url = process.env.NEXT_PUBLIC_SUPABASE_URL || "";
  const supabase_anon_key = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || "";

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
