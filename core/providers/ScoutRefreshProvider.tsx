"use client";

import { useScoutRefresh } from "../hooks/useScoutRefresh";
import { useScoutDbListener } from "../hooks/useScoutDbListener";
import { createContext, useContext, useMemo, ReactNode } from "react";
import { createBrowserClient } from "@supabase/ssr";
import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";

// Create context for the Supabase client
const SupabaseContext = createContext<SupabaseClient<Database> | null>(null);

// Create context for connection status
interface ConnectionStatus {
  isConnected: boolean;
  isConnecting: boolean;
  lastError: string | null;
  retryCount: number;
  reconnect: () => void;
}

const ConnectionStatusContext = createContext<ConnectionStatus | null>(null);

// Hook to use the Supabase client
export function useSupabase() {
  const supabase = useContext(SupabaseContext);
  if (!supabase) {
    throw new Error("useSupabase must be used within a SupabaseProvider");
  }
  return supabase;
}

// Hook to use connection status
export function useConnectionStatus() {
  const connectionStatus = useContext(ConnectionStatusContext);
  if (!connectionStatus) {
    throw new Error(
      "useConnectionStatus must be used within a ScoutRefreshProvider"
    );
  }
  return connectionStatus;
}

export interface ScoutRefreshProviderProps {
  children: ReactNode;
}

export function ScoutRefreshProvider({ children }: ScoutRefreshProviderProps) {
  const url = process.env.NEXT_PUBLIC_SUPABASE_URL || "";
  const anon_key = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || "";

  // Create a single Supabase client instance using useMemo to prevent recreation
  const supabaseClient = useMemo(() => {
    console.log("[ScoutRefreshProvider] Creating Supabase client");
    return createBrowserClient<Database>(url, anon_key);
  }, [url, anon_key]);

  // Use the enhanced DB listener with connection status
  useScoutDbListener(supabaseClient);
  useScoutRefresh();

  // // Log connection status changes for debugging
  // if (connectionStatus.lastError) {
  //   console.warn(
  //     "[ScoutRefreshProvider] DB Listener error:",
  //     connectionStatus.lastError
  //   );
  // }

  // if (connectionStatus.isConnected) {
  //   console.log("[ScoutRefreshProvider] ✅ DB Listener connected");
  // }

  return (
    <SupabaseContext.Provider value={supabaseClient}>
      {children}
    </SupabaseContext.Provider>
  );
}
