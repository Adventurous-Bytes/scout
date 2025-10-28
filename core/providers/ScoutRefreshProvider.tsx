"use client";

import { useScoutRefresh } from "../hooks/useScoutRefresh";
import { createContext, useContext, useMemo, ReactNode, useRef } from "react";
import { createBrowserClient } from "@supabase/ssr";
import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { useScoutRealtimeConnectivity } from "../hooks/useScoutRealtimeConnectivity";
import { useScoutRealtimeDevices } from "../hooks/useScoutRealtimeDevices";

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
      "useConnectionStatus must be used within a ScoutRefreshProvider",
    );
  }
  return connectionStatus;
}

export interface ScoutRefreshProviderProps {
  children: ReactNode;
}

export function ScoutRefreshProvider({ children }: ScoutRefreshProviderProps) {
  // Use refs to store the URL and key to prevent unnecessary recreations
  const urlRef = useRef(process.env.NEXT_PUBLIC_SUPABASE_URL || "");
  const anonKeyRef = useRef(process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || "");

  // Create a single Supabase client instance that only runs once
  const supabaseClient = useMemo(() => {
    console.log("[ScoutRefreshProvider] Creating Supabase client");
    return createBrowserClient<Database>(urlRef.current, anonKeyRef.current);
  }, []); // Empty dependency array ensures this only runs once

  // Use the enhanced DB listener with connection status
  useScoutRealtimeConnectivity(supabaseClient);
  useScoutRealtimeDevices(supabaseClient);
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
