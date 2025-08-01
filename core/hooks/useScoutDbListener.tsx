"use client";

import { useAppDispatch } from "../store/hooks";
import { useEffect, useRef, useState } from "react";
import {
  addDevice,
  addPlan,
  addTag,
  deleteDevice,
  deletePlan,
  deleteTag,
  updateDevice,
  updatePlan,
  updateTag,
} from "../store/scout";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";

// Connection state enum
enum ConnectionState {
  DISCONNECTED = "disconnected",
  CONNECTING = "connecting",
  CONNECTED = "connected",
  ERROR = "error",
}

/**
 * Hook for listening to real-time database changes
 */
export function useScoutDbListener(scoutSupabase: SupabaseClient<Database>) {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();
  const [connectionState, setConnectionState] = useState<ConnectionState>(
    ConnectionState.DISCONNECTED
  );
  const [lastError, setLastError] = useState<string | null>(null);

  // Clean up all channels
  const cleanupChannels = () => {
    channels.current.forEach((channel) => {
      if (channel && scoutSupabase) {
        try {
          scoutSupabase.removeChannel(channel);
        } catch (error) {
          console.warn("[DB Listener] Error removing channel:", error);
        }
      }
    });
    channels.current = [];
  };

  // Create event handlers
  const handlers = {
    tags: {
      INSERT: (payload: any) => {
        if (payload.new) dispatch(addTag(payload.new));
      },
      UPDATE: (payload: any) => {
        if (payload.new) dispatch(updateTag(payload.new));
      },
      DELETE: (payload: any) => {
        if (payload.old) dispatch(deleteTag(payload.old));
      },
    },
    devices: {
      INSERT: (payload: any) => {
        if (payload.new) dispatch(addDevice(payload.new));
      },
      UPDATE: (payload: any) => {
        if (payload.new) dispatch(updateDevice(payload.new));
      },
      DELETE: (payload: any) => {
        if (payload.old) dispatch(deleteDevice(payload.old));
      },
    },
    plans: {
      INSERT: (payload: any) => {
        if (payload.new) dispatch(addPlan(payload.new));
      },
      UPDATE: (payload: any) => {
        if (payload.new) dispatch(updatePlan(payload.new));
      },
      DELETE: (payload: any) => {
        if (payload.old) dispatch(deletePlan(payload.old));
      },
    },
    sessions: {
      INSERT: (payload: any) =>
        console.log("[DB Listener] Session INSERT:", payload),
      UPDATE: (payload: any) =>
        console.log("[DB Listener] Session UPDATE:", payload),
      DELETE: (payload: any) =>
        console.log("[DB Listener] Session DELETE:", payload),
    },
    connectivity: {
      INSERT: (payload: any) =>
        console.log("[DB Listener] Connectivity INSERT:", payload),
      UPDATE: (payload: any) =>
        console.log("[DB Listener] Connectivity UPDATE:", payload),
      DELETE: (payload: any) =>
        console.log("[DB Listener] Connectivity DELETE:", payload),
    },
  };

  // Set up channels
  const setupChannels = async (): Promise<boolean> => {
    if (!scoutSupabase) return false;

    cleanupChannels();

    const tables = Object.keys(handlers) as Array<keyof typeof handlers>;
    let successCount = 0;
    const totalChannels = tables.length;

    for (const tableName of tables) {
      try {
        const channelName = `scout_broadcast_${tableName}_${Date.now()}`;
        const channel = scoutSupabase.channel(channelName, {
          config: { private: false },
        });

        // Set up event handlers
        const tableHandler = handlers[tableName];
        Object.entries(tableHandler).forEach(([event, handler]) => {
          channel.on("broadcast", { event }, handler);
        });

        // Subscribe to the channel
        channel.subscribe((status: string) => {
          if (status === "SUBSCRIBED") {
            successCount++;
            if (successCount === totalChannels) {
              setConnectionState(ConnectionState.CONNECTED);
              setLastError(null);
            }
          } else if (status === "CHANNEL_ERROR" || status === "TIMED_OUT") {
            setLastError(`Channel subscription failed: ${status}`);
          }
        });

        channels.current.push(channel);
      } catch (error) {
        console.error(
          `[DB Listener] Failed to set up ${tableName} channel:`,
          error
        );
      }
    }

    return successCount > 0;
  };

  // Initialize connection
  const initializeConnection = async () => {
    if (!scoutSupabase) return;

    setConnectionState(ConnectionState.CONNECTING);

    try {
      // Test database connection
      const { error } = await scoutSupabase.from("tags").select("id").limit(1);
      if (error) {
        throw new Error("Database connection test failed");
      }

      // Set up channels
      const success = await setupChannels();
      if (!success) {
        throw new Error("Channel setup failed");
      }
    } catch (error) {
      setLastError(error instanceof Error ? error.message : "Unknown error");
      setConnectionState(ConnectionState.ERROR);
    }
  };

  // Main effect
  useEffect(() => {
    if (!scoutSupabase) {
      setConnectionState(ConnectionState.ERROR);
      setLastError("No Supabase client available");
      return;
    }

    initializeConnection();

    return () => {
      cleanupChannels();
    };
  }, [scoutSupabase]);

  return {
    connectionState,
    lastError,
    isConnected: connectionState === ConnectionState.CONNECTED,
    isConnecting: connectionState === ConnectionState.CONNECTING,
  };
}
