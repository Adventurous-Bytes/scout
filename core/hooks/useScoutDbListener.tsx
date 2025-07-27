"use client";

import { useAppDispatch } from "../store/hooks";
import { useEffect, useRef } from "react";
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
import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";

export function useScoutDbListener(scoutSupabase: SupabaseClient<Database>) {
  const supabase = useRef<any>(null);
  const channels = useRef<any[]>([]);
  const dispatch = useAppDispatch();

  function handleTagInserts(payload: any) {
    console.log("[DB Listener] Tag INSERT received:", payload);
    // Broadcast payload contains the record directly
    const tagData = payload.new || payload;
    dispatch(addTag(tagData));
  }

  function handleTagDeletes(payload: any) {
    console.log("[DB Listener] Tag DELETE received:", payload);
    console.log("[DB Listener] Tag DELETE - payload structure:", {
      hasOld: !!payload.old,
      oldId: payload.old?.id,
      oldEventId: payload.old?.event_id,
      oldClassName: payload.old?.class_name,
      fullPayload: payload,
    });

    // Broadcast payload contains the old record
    const tagData = payload.old || payload;
    if (!tagData || !tagData.id) {
      console.error(
        "[DB Listener] Tag DELETE - Invalid payload, missing tag data"
      );
      return;
    }

    console.log(
      "[DB Listener] Tag DELETE - Dispatching deleteTag action with ID:",
      tagData.id
    );
    dispatch(deleteTag(tagData));
  }

  function handleTagUpdates(payload: any) {
    console.log("[DB Listener] Tag UPDATE received:", payload);
    // Broadcast payload contains the new record
    const tagData = payload.new || payload;
    dispatch(updateTag(tagData));
  }

  async function handleDeviceInserts(payload: any) {
    console.log("[DB Listener] Device INSERT received:", payload);
    // Broadcast payload contains the record directly
    const deviceData = payload.new || payload;
    dispatch(addDevice(deviceData));
  }

  function handleDeviceDeletes(payload: any) {
    console.log("[DB Listener] Device DELETE received:", payload);
    // Broadcast payload contains the old record
    const deviceData = payload.old || payload;
    dispatch(deleteDevice(deviceData));
  }

  async function handleDeviceUpdates(payload: any) {
    console.log("[DB Listener] Device UPDATE received:", payload);
    // Broadcast payload contains the new record
    const deviceData = payload.new || payload;
    dispatch(updateDevice(deviceData));
  }

  function handlePlanInserts(payload: any) {
    console.log("[DB Listener] Plan INSERT received:", payload);
    // Broadcast payload contains the record directly
    const planData = payload.new || payload;
    dispatch(addPlan(planData));
  }

  function handlePlanDeletes(payload: any) {
    console.log("[DB Listener] Plan DELETE received:", payload);
    // Broadcast payload contains the old record
    const planData = payload.old || payload;
    dispatch(deletePlan(planData));
  }

  function handlePlanUpdates(payload: any) {
    console.log("[DB Listener] Plan UPDATE received:", payload);
    // Broadcast payload contains the new record
    const planData = payload.new || payload;
    dispatch(updatePlan(planData));
  }

  function handleSessionInserts(payload: any) {
    console.log("[DB Listener] Session INSERT received:", payload);
    // Broadcast payload contains the record directly
    const sessionData = payload.new || payload;
    dispatch(addSessionToStore(sessionData));
  }

  function handleSessionDeletes(payload: any) {
    console.log("[DB Listener] Session DELETE received:", payload);
    // Broadcast payload contains the old record
    const sessionData = payload.old || payload;
    dispatch(deleteSessionFromStore(sessionData));
  }

  function handleSessionUpdates(payload: any) {
    console.log("[DB Listener] Session UPDATE received:", payload);
    // Broadcast payload contains the new record
    const sessionData = payload.new || payload;
    dispatch(updateSessionInStore(sessionData));
  }

  function handleConnectivityInserts(payload: any) {
    console.log("[DB Listener] Connectivity INSERT received:", payload);
    // For now, we'll just log connectivity changes since they're related to sessions
    // In the future, we might want to update session connectivity data
  }

  function handleConnectivityDeletes(payload: any) {
    console.log("[DB Listener] Connectivity DELETE received:", payload);
    // For now, we'll just log connectivity changes since they're related to sessions
    // In the future, we might want to update session connectivity data
  }

  function handleConnectivityUpdates(payload: any) {
    console.log("[DB Listener] Connectivity UPDATE received:", payload);
    // For now, we'll just log connectivity changes since they're related to sessions
    // In the future, we might want to update session connectivity data
  }

  useEffect(() => {
    console.log("=== SCOUT DB LISTENER DEBUG ===");
    console.log(
      "[DB Listener] Using shared Supabase client from ScoutRefreshProvider context"
    );

    if (!scoutSupabase) {
      console.error(
        "[DB Listener] No Supabase client available from ScoutRefreshProvider context"
      );
      return;
    }

    supabase.current = scoutSupabase;

    // Test authentication first
    const testAuth = async () => {
      try {
        const {
          data: { user },
          error,
        } = await scoutSupabase.auth.getUser();
        console.log(
          "[DB Listener] Auth test - User:",
          user ? "authenticated" : "anonymous"
        );
        console.log("[DB Listener] Auth test - Error:", error);
      } catch (err) {
        console.warn("[DB Listener] Auth test failed:", err);
      }
    };
    testAuth();

    // Set up authentication for Realtime Authorization (required for broadcast)
    const setupRealtimeAuth = async () => {
      try {
        await scoutSupabase.realtime.setAuth();
        console.log(
          "[DB Listener] ✅ Realtime authentication set up successfully"
        );
      } catch (err) {
        console.warn(
          "[DB Listener] ❌ Failed to set up realtime authentication:",
          err
        );
      }
    };
    setupRealtimeAuth();

    // Create channels for each table using broadcast
    const createBroadcastChannel = (tableName: string) => {
      const channelName = `scout_broadcast_${tableName}_${Date.now()}`;
      console.log(
        `[DB Listener] Creating broadcast channel for ${tableName}:`,
        channelName
      );

      return scoutSupabase.channel(channelName, {
        config: { private: true }, // Required for broadcast with Realtime Authorization
      });
    };

    // Plans channel
    const plansChannel = createBroadcastChannel("plans");
    plansChannel
      .on("broadcast", { event: "INSERT" }, (payload) => {
        console.log("[DB Listener] Plans INSERT received:", payload);
        handlePlanInserts(payload);
      })
      .on("broadcast", { event: "UPDATE" }, (payload) => {
        console.log("[DB Listener] Plans UPDATE received:", payload);
        handlePlanUpdates(payload);
      })
      .on("broadcast", { event: "DELETE" }, (payload) => {
        console.log("[DB Listener] Plans DELETE received:", payload);
        handlePlanDeletes(payload);
      })
      .subscribe((status: any) => {
        console.log(`[DB Listener] Plans channel status:`, status);
      });

    // Devices channel
    const devicesChannel = createBroadcastChannel("devices");
    devicesChannel
      .on("broadcast", { event: "INSERT" }, (payload) => {
        console.log("[DB Listener] Devices INSERT received:", payload);
        handleDeviceInserts(payload);
      })
      .on("broadcast", { event: "UPDATE" }, (payload) => {
        console.log("[DB Listener] Devices UPDATE received:", payload);
        handleDeviceUpdates(payload);
      })
      .on("broadcast", { event: "DELETE" }, (payload) => {
        console.log("[DB Listener] Devices DELETE received:", payload);
        handleDeviceDeletes(payload);
      })
      .subscribe((status: any) => {
        console.log(`[DB Listener] Devices channel status:`, status);
      });

    // Tags channel
    const tagsChannel = createBroadcastChannel("tags");
    tagsChannel
      .on("broadcast", { event: "INSERT" }, (payload) => {
        console.log("[DB Listener] Tags INSERT received:", payload);
        handleTagInserts(payload);
      })
      .on("broadcast", { event: "UPDATE" }, (payload) => {
        console.log("[DB Listener] Tags UPDATE received:", payload);
        handleTagUpdates(payload);
      })
      .on("broadcast", { event: "DELETE" }, (payload) => {
        console.log("[DB Listener] Tags DELETE received:", payload);
        handleTagDeletes(payload);
      })
      .subscribe((status: any) => {
        console.log(`[DB Listener] Tags channel status:`, status);
      });

    // Sessions channel
    const sessionsChannel = createBroadcastChannel("sessions");
    sessionsChannel
      .on("broadcast", { event: "INSERT" }, (payload) => {
        console.log("[DB Listener] Sessions INSERT received:", payload);
        handleSessionInserts(payload);
      })
      .on("broadcast", { event: "UPDATE" }, (payload) => {
        console.log("[DB Listener] Sessions UPDATE received:", payload);
        handleSessionUpdates(payload);
      })
      .on("broadcast", { event: "DELETE" }, (payload) => {
        console.log("[DB Listener] Sessions DELETE received:", payload);
        handleSessionDeletes(payload);
      })
      .subscribe((status: any) => {
        console.log(`[DB Listener] Sessions channel status:`, status);
      });

    // Connectivity channel
    const connectivityChannel = createBroadcastChannel("connectivity");
    connectivityChannel
      .on("broadcast", { event: "INSERT" }, (payload) => {
        console.log("[DB Listener] Connectivity INSERT received:", payload);
        handleConnectivityInserts(payload);
      })
      .on("broadcast", { event: "UPDATE" }, (payload) => {
        console.log("[DB Listener] Connectivity UPDATE received:", payload);
        handleConnectivityUpdates(payload);
      })
      .on("broadcast", { event: "DELETE" }, (payload) => {
        console.log("[DB Listener] Connectivity DELETE received:", payload);
        handleConnectivityDeletes(payload);
      })
      .subscribe((status: any) => {
        console.log(`[DB Listener] Connectivity channel status:`, status);
      });

    // Add all channels to the channels array
    channels.current.push(
      plansChannel,
      devicesChannel,
      tagsChannel,
      sessionsChannel,
      connectivityChannel
    );

    // Test the connection with system events
    const testChannelName = `test_connection_${Date.now()}`;
    console.log("[DB Listener] Creating test channel:", testChannelName);
    const testChannel = scoutSupabase.channel(testChannelName);
    testChannel
      .on("system", { event: "disconnect" }, () => {
        console.log("[DB Listener] 🔌 Disconnected from Supabase");
      })
      .on("system", { event: "reconnect" }, () => {
        console.log("[DB Listener] 🔗 Reconnected to Supabase");
      })
      .on("system", { event: "error" }, (error: any) => {
        console.warn("[DB Listener] ❌ System error:", error);
      })
      .subscribe((status: any) => {
        console.log("[DB Listener] Test channel status:", status);
      });

    channels.current.push(testChannel);

    // Test a simple database query to verify connection
    const testDbConnection = async () => {
      try {
        const { data, error } = await scoutSupabase
          .from("tags")
          .select("count")
          .limit(1);
        console.log("[DB Listener] DB connection test - Success:", !!data);
        console.log("[DB Listener] DB connection test - Error:", error);
      } catch (err) {
        console.error("[DB Listener] DB connection test failed:", err);
      }
    };
    testDbConnection();

    console.log("=== END SCOUT DB LISTENER DEBUG ===");

    // Cleanup function
    return () => {
      console.log("[DB Listener] 🧹 Cleaning up channels");
      channels.current.forEach((channel) => {
        if (channel) {
          scoutSupabase.removeChannel(channel);
        }
      });
      channels.current = [];
    };
  }, [scoutSupabase, dispatch]);
}
