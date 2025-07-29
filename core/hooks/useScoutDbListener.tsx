"use client";

import { useAppDispatch } from "../store/hooks";
import { useEffect, useRef, useState, useCallback } from "react";
import {
  addDevice,
  addPlan,
  addTag,
  addSessionToStore,
  deleteDevice,
  deletePlan,
  deleteSessionFromStore,
  deleteTag,
  updateDevice,
  updatePlan,
  updateSessionInStore,
  updateTag,
} from "../store/scout";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";

// Define explicit types for broadcast payloads
interface BroadcastPayload<T = unknown> {
  new?: T;
  old?: T;
  event: string;
  type: "broadcast";
  [key: string]: unknown;
}

// Define types for each table's data
type PlanData = Database["public"]["Tables"]["plans"]["Row"];
type DeviceData = Database["public"]["Tables"]["devices"]["Row"];
type TagData = Database["public"]["Tables"]["tags"]["Row"];
type SessionData = Database["public"]["Tables"]["sessions"]["Row"];
type ConnectivityData = Database["public"]["Tables"]["connectivity"]["Row"];

// Connection state enum
enum ConnectionState {
  DISCONNECTED = "disconnected",
  CONNECTING = "connecting",
  CONNECTED = "connected",
  ERROR = "error",
}

// Reconnection configuration
const RECONNECTION_CONFIG = {
  MAX_RETRIES: 10,
  INITIAL_DELAY: 1000, // 1 second
  MAX_DELAY: 30000, // 30 seconds
  BACKOFF_MULTIPLIER: 2,
  JITTER_FACTOR: 0.1, // 10% jitter
};

/**
 * Hook for listening to real-time database changes with robust disconnect handling.
 *
 * Features:
 * - Automatic reconnection with exponential backoff
 * - Connection state tracking
 * - Error handling and retry logic
 * - Manual reconnection capability
 *
 * @param scoutSupabase - The Supabase client instance
 * @returns Connection status and control functions
 */
export function useScoutDbListener(scoutSupabase: SupabaseClient<Database>) {
  const supabase = useRef<SupabaseClient<Database> | null>(null);
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();

  // Connection state management
  const [connectionState, setConnectionState] = useState<ConnectionState>(
    ConnectionState.DISCONNECTED
  );
  const [lastError, setLastError] = useState<string | null>(null);
  const [retryCount, setRetryCount] = useState(0);

  // Reconnection management
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const isInitializingRef = useRef(false);
  const isDestroyedRef = useRef(false);

  // Calculate exponential backoff delay with jitter
  const calculateBackoffDelay = useCallback((attempt: number): number => {
    const baseDelay = Math.min(
      RECONNECTION_CONFIG.INITIAL_DELAY *
        Math.pow(RECONNECTION_CONFIG.BACKOFF_MULTIPLIER, attempt),
      RECONNECTION_CONFIG.MAX_DELAY
    );

    const jitter =
      baseDelay * RECONNECTION_CONFIG.JITTER_FACTOR * (Math.random() - 0.5);
    return Math.max(100, baseDelay + jitter); // Minimum 100ms delay
  }, []);

  // Clean up all channels
  const cleanupChannels = useCallback(() => {
    console.log("[DB Listener] 🧹 Cleaning up channels");
    channels.current.forEach((channel) => {
      if (channel && supabase.current) {
        try {
          supabase.current.removeChannel(channel);
        } catch (error) {
          console.warn("[DB Listener] Error removing channel:", error);
        }
      }
    });
    channels.current = [];
  }, []);

  // Cancel any pending reconnection attempts
  const cancelReconnection = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  }, []);

  // Test database connection
  const testDbConnection = useCallback(async (): Promise<boolean> => {
    if (!supabase.current) return false;

    try {
      const { data, error } = await supabase.current
        .from("tags")
        .select("count")
        .limit(1);

      if (error) {
        console.warn("[DB Listener] DB connection test failed:", error);
        return false;
      }

      console.log("[DB Listener] ✅ DB connection test successful");
      return true;
    } catch (err) {
      console.error("[DB Listener] DB connection test failed:", err);
      return false;
    }
  }, []);

  // Set up realtime authentication
  const setupRealtimeAuth = useCallback(async (): Promise<boolean> => {
    if (!supabase.current) return false;

    try {
      await supabase.current.realtime.setAuth();
      console.log(
        "[DB Listener] ✅ Realtime authentication set up successfully"
      );
      return true;
    } catch (err) {
      console.warn(
        "[DB Listener] ❌ Failed to set up realtime authentication:",
        err
      );
      return false;
    }
  }, []);

  // Generic event handler factory
  const createEventHandler = useCallback(
    <T,>(
      action: (data: T) => void,
      dataKey: "new" | "old",
      entityName: string
    ) => {
      return (payload: BroadcastPayload<T>) => {
        console.log(
          `[DB Listener] ${entityName} ${payload.event} received:`,
          payload
        );
        const data = payload[dataKey];
        if (!data) {
          console.error(
            `[DB Listener] ${entityName} ${payload.event} - Invalid payload, missing ${dataKey} data`
          );
          return;
        }
        action(data);
      };
    },
    []
  );

  // Create event handlers using the factory
  const handlers = useCallback(
    () => ({
      tags: {
        INSERT: createEventHandler(dispatch.bind(null, addTag), "new", "Tag"),
        UPDATE: createEventHandler(
          dispatch.bind(null, updateTag),
          "new",
          "Tag"
        ),
        DELETE: createEventHandler(
          dispatch.bind(null, deleteTag),
          "old",
          "Tag"
        ),
      },
      devices: {
        INSERT: createEventHandler(
          dispatch.bind(null, addDevice),
          "new",
          "Device"
        ),
        UPDATE: createEventHandler(
          dispatch.bind(null, updateDevice),
          "new",
          "Device"
        ),
        DELETE: createEventHandler(
          dispatch.bind(null, deleteDevice),
          "old",
          "Device"
        ),
      },
      plans: {
        INSERT: createEventHandler(dispatch.bind(null, addPlan), "new", "Plan"),
        UPDATE: createEventHandler(
          dispatch.bind(null, updatePlan),
          "new",
          "Plan"
        ),
        DELETE: createEventHandler(
          dispatch.bind(null, deletePlan),
          "old",
          "Plan"
        ),
      },
      sessions: {
        INSERT: createEventHandler(
          dispatch.bind(null, addSessionToStore),
          "new",
          "Session"
        ),
        UPDATE: createEventHandler(
          dispatch.bind(null, updateSessionInStore),
          "new",
          "Session"
        ),
        DELETE: createEventHandler(
          dispatch.bind(null, deleteSessionFromStore),
          "old",
          "Session"
        ),
      },
      connectivity: {
        INSERT: (payload: BroadcastPayload<ConnectivityData>) =>
          console.log("[DB Listener] Connectivity INSERT received:", payload),
        UPDATE: (payload: BroadcastPayload<ConnectivityData>) =>
          console.log("[DB Listener] Connectivity UPDATE received:", payload),
        DELETE: (payload: BroadcastPayload<ConnectivityData>) =>
          console.log("[DB Listener] Connectivity DELETE received:", payload),
      },
    }),
    [createEventHandler, dispatch]
  );

  // Create a channel with proper error handling
  const createChannel = useCallback(
    (tableName: string): RealtimeChannel | null => {
      if (!supabase.current) return null;

      const channelName = `scout_broadcast_${tableName}_${Date.now()}`;
      console.log(
        `[DB Listener] Creating broadcast channel for ${tableName}:`,
        channelName
      );

      try {
        const channel = supabase.current.channel(channelName, {
          config: { private: true },
        });

        // Add system event handlers for connection monitoring
        channel
          .on("system", { event: "disconnect" }, () => {
            console.log(`[DB Listener] 🔌 ${tableName} channel disconnected`);
            setConnectionState(ConnectionState.DISCONNECTED);
            setLastError("Channel disconnected");
          })
          .on("system", { event: "reconnect" }, () => {
            console.log(`[DB Listener] 🔗 ${tableName} channel reconnected`);
          })
          .on("system", { event: "error" }, (error: unknown) => {
            console.warn(`[DB Listener] ❌ ${tableName} channel error:`, error);
            setLastError(`Channel error: ${error}`);
          });

        return channel;
      } catch (error) {
        console.error(
          `[DB Listener] Failed to create ${tableName} channel:`,
          error
        );
        return null;
      }
    },
    []
  );

  // Set up all channels
  const setupChannels = useCallback(async (): Promise<boolean> => {
    if (!supabase.current) return false;

    cleanupChannels();

    const tableHandlers = handlers();
    const tables = Object.keys(tableHandlers) as Array<
      keyof typeof tableHandlers
    >;
    let successCount = 0;
    const totalChannels = tables.length;

    for (const tableName of tables) {
      const channel = createChannel(tableName);
      if (!channel) continue;

      try {
        // Set up event handlers
        const tableHandler = tableHandlers[tableName];
        Object.entries(tableHandler).forEach(([event, handler]) => {
          channel.on("broadcast", { event }, handler);
        });

        // Subscribe to the channel
        channel.subscribe((status: string) => {
          console.log(`[DB Listener] ${tableName} channel status:`, status);

          if (status === "SUBSCRIBED") {
            successCount++;
            if (successCount === totalChannels) {
              setConnectionState(ConnectionState.CONNECTED);
              setRetryCount(0);
              setLastError(null);
              console.log(
                "[DB Listener] ✅ All channels successfully subscribed"
              );
            }
          } else if (status === "CHANNEL_ERROR" || status === "TIMED_OUT") {
            console.error(
              `[DB Listener] ${tableName} channel failed to subscribe:`,
              status
            );
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
  }, [cleanupChannels, createChannel, handlers]);

  // Schedule reconnection with exponential backoff
  const scheduleReconnection = useCallback(() => {
    if (
      isDestroyedRef.current ||
      retryCount >= RECONNECTION_CONFIG.MAX_RETRIES
    ) {
      console.log(
        "[DB Listener] Max reconnection attempts reached or hook destroyed"
      );
      setConnectionState(ConnectionState.ERROR);
      return;
    }

    const delay = calculateBackoffDelay(retryCount);
    console.log(
      `[DB Listener] Scheduling reconnection attempt ${
        retryCount + 1
      } in ${delay}ms`
    );

    reconnectTimeoutRef.current = setTimeout(() => {
      if (!isDestroyedRef.current) {
        initializeConnection();
      }
    }, delay);
  }, [retryCount, calculateBackoffDelay]);

  // Initialize connection
  const initializeConnection = useCallback(async () => {
    if (isDestroyedRef.current || isInitializingRef.current) return;

    isInitializingRef.current = true;
    setConnectionState(ConnectionState.CONNECTING);

    try {
      console.log("[DB Listener] 🔄 Initializing connection...");

      // Test database connection
      const dbConnected = await testDbConnection();
      if (!dbConnected) {
        throw new Error("Database connection test failed");
      }

      // Set up realtime authentication
      const authSuccess = await setupRealtimeAuth();
      if (!authSuccess) {
        throw new Error("Realtime authentication failed");
      }

      // Set up channels
      const channelsSuccess = await setupChannels();
      if (!channelsSuccess) {
        throw new Error("Channel setup failed");
      }

      console.log("[DB Listener] ✅ Connection initialized successfully");
    } catch (error) {
      console.error(
        "[DB Listener] ❌ Connection initialization failed:",
        error
      );
      setLastError(error instanceof Error ? error.message : "Unknown error");
      setConnectionState(ConnectionState.ERROR);
      setRetryCount((prev) => prev + 1);

      // Schedule reconnection
      scheduleReconnection();
    } finally {
      isInitializingRef.current = false;
    }
  }, [
    testDbConnection,
    setupRealtimeAuth,
    setupChannels,
    scheduleReconnection,
  ]);

  // Manual reconnection function
  const reconnect = useCallback(() => {
    if (isDestroyedRef.current) return;

    console.log("[DB Listener] 🔄 Manual reconnection requested");
    cancelReconnection();
    setRetryCount(0);
    setLastError(null);
    initializeConnection();
  }, [cancelReconnection, initializeConnection]);

  // Main effect
  useEffect(() => {
    console.log("=== SCOUT DB LISTENER INITIALIZATION ===");

    if (!scoutSupabase) {
      console.error("[DB Listener] No Supabase client available");
      setConnectionState(ConnectionState.ERROR);
      setLastError("No Supabase client available");
      return;
    }

    supabase.current = scoutSupabase;
    isDestroyedRef.current = false;

    // Initialize connection
    initializeConnection();

    // Cleanup function
    return () => {
      console.log("[DB Listener] 🧹 Cleaning up hook");
      isDestroyedRef.current = true;
      cancelReconnection();
      cleanupChannels();
    };
  }, [
    scoutSupabase,
    initializeConnection,
    cancelReconnection,
    cleanupChannels,
  ]);

  // Return connection state and manual reconnect function
  return {
    connectionState,
    lastError,
    retryCount,
    reconnect,
    isConnected: connectionState === ConnectionState.CONNECTED,
    isConnecting: connectionState === ConnectionState.CONNECTING,
  };
}

/**
 * Return type for useScoutDbListener hook
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const {
 *     isConnected,
 *     isConnecting,
 *     lastError,
 *     retryCount,
 *     reconnect
 *   } = useConnectionStatus();
 *
 *   if (isConnecting) {
 *     return <div>Connecting to database...</div>;
 *   }
 *
 *   if (lastError) {
 *     return (
 *       <div>
 *         <p>Connection error: {lastError}</p>
 *         <p>Retry attempts: {retryCount}</p>
 *         <button onClick={reconnect}>Reconnect</button>
 *       </div>
 *     );
 *   }
 *
 *   if (!isConnected) {
 *     return <div>Disconnected from database</div>;
 *   }
 *
 *   return <div>Connected to database</div>;
 * }
 * ```
 */
