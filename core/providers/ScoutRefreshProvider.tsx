"use client";

import { useScoutRefresh } from "../hooks/useScoutRefresh";
import { useScoutDbListener } from "../hooks/useScoutDbListener";
import { createContext, useContext, useRef, ReactNode } from "react";
import { createBrowserClient } from "@supabase/ssr";
import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";

// Create context for the Supabase client
const SupabaseContext = createContext<SupabaseClient<Database> | null>(null);

// Hook to use the Supabase client
export function useSupabase() {
  const supabase = useContext(SupabaseContext);
  if (!supabase) {
    throw new Error("useSupabase must be used within a SupabaseProvider");
  }
  return supabase;
}

export interface ScoutRefreshProviderProps {
  children: ReactNode;
}

export function ScoutRefreshProvider({ children }: ScoutRefreshProviderProps) {
  const url = process.env.NEXT_PUBLIC_SUPABASE_URL || "";
  const anon_key = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || "";

  // Create a single Supabase client instance
  const supabaseRef = useRef<SupabaseClient<Database> | null>(null);

  if (!supabaseRef.current) {
    supabaseRef.current = createBrowserClient<Database>(url, anon_key);
    console.log("[ScoutRefreshProvider] Created Supabase client");
  }

  useScoutDbListener(supabaseRef.current);
  useScoutRefresh();

  return (
    <SupabaseContext.Provider value={supabaseRef.current}>
      {children}
    </SupabaseContext.Provider>
  );
}
