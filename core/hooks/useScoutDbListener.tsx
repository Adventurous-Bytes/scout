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
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const maxReconnectAttempts = 10;
  const baseDelay = 1000; // 1 second
  const maxDelay = 5000; // 5 seconds
  const lastChannelIdRef = useRef<string | null>(null);

  function handleTagInserts(payload: any) {
    console.log("[DB Listener] Tag INSERT received:", payload.new);
    dispatch(addTag(payload.new));
  }

  function handleTagDeletes(payload: any) {
    console.log("[DB Listener] Tag DELETE received:", payload.old);
    if (!payload.old || !payload.old.id) {
      console.error(
        "[DB Listener] Tag DELETE - Invalid payload, missing tag data"
      );
      return;
    }
    dispatch(deleteTag(payload.old));
  }

  function handleTagUpdates(payload: any) {
    console.log("[DB Listener] Tag UPDATE received:", payload.new);
    dispatch(updateTag(payload.new));
  }

  function handleDeviceInserts(payload: any) {
    console.log("[DB Listener] Device INSERT received:", payload.new);
    dispatch(addDevice(payload.new));
  }

  function handleDeviceDeletes(payload: any) {
    console.log("[DB Listener] Device DELETE received:", payload.old);
    dispatch(deleteDevice(payload.old));
  }

  function handleDeviceUpdates(payload: any) {
    console.log("[DB Listener] Device UPDATE received:", payload.new);
    dispatch(updateDevice(payload.new));
  }

  function handlePlanInserts(payload: any) {
    console.log("[DB Listener] Plan INSERT received:", payload.new);
    dispatch(addPlan(payload.new));
  }

  function handlePlanDeletes(payload: any) {
    console.log("[DB Listener] Plan DELETE received:", payload.old);
    dispatch(deletePlan(payload.old));
  }

  function handlePlanUpdates(payload: any) {
    console.log("[DB Listener] Plan UPDATE received:", payload.new);
    dispatch(updatePlan(payload.new));
  }

  function handleSessionInserts(payload: any) {
    console.log("[DB Listener] Session INSERT received:", payload.new);
    dispatch(addSessionToStore(payload.new));
  }

  function handleSessionDeletes(payload: any) {
    console.log("[DB Listener] Session DELETE received:", payload.old);
    if (!payload.old || !payload.old.id) {
      console.error(
        "[DB Listener] Session DELETE - Invalid payload, missing session data"
      );
      return;
    }
    dispatch(deleteSessionFromStore(payload.old));
  }

  function handleSessionUpdates(payload: any) {
    console.log("[DB Listener] Session UPDATE received:", payload.new);
    dispatch(updateSessionInStore(payload.new));
  }

  function handleConnectivityInserts(payload: any) {
    console.log("[DB Listener] Connectivity INSERT received:", payload.new);
  }

  function handleConnectivityDeletes(payload: any) {
    console.log("[DB Listener] Connectivity DELETE received:", payload.old);
  }

  function handleConnectivityUpdates(payload: any) {
    console.log("[DB Listener] Connectivity UPDATE received:", payload.new);
  }

  // Clean up all channels
  const cleanupChannels = () => {
    channels.current.forEach((channel) => {
      if (channel) {
        scoutSupabase.removeChannel(channel);
      }
    });
    channels.current = [];
  };

  // Setup channel with event handlers
  const setupChannel = () => {
    if (!scoutSupabase) return null;

    const channelId = `scout_realtime_${Date.now()}_${Math.random()
      .toString(36)
      .substr(2, 9)}`;
    const mainChannel = scoutSupabase.channel(channelId);
    lastChannelIdRef.current = channelId;

    console.log(`[DB Listener] Creating channel: ${channelId}`);

    // Subscribe to all events
    mainChannel
      .on(
        "postgres_changes",
        { event: "INSERT", schema: "public", table: "plans" },
        handlePlanInserts
      )
      .on(
        "postgres_changes",
        { event: "DELETE", schema: "public", table: "plans" },
        handlePlanDeletes
      )
      .on(
        "postgres_changes",
        { event: "UPDATE", schema: "public", table: "plans" },
        handlePlanUpdates
      )
      .on(
        "postgres_changes",
        { event: "INSERT", schema: "public", table: "devices" },
        handleDeviceInserts
      )
      .on(
        "postgres_changes",
        { event: "DELETE", schema: "public", table: "devices" },
        handleDeviceDeletes
      )
      .on(
        "postgres_changes",
        { event: "UPDATE", schema: "public", table: "devices" },
        handleDeviceUpdates
      )
      .on(
        "postgres_changes",
        { event: "INSERT", schema: "public", table: "tags" },
        handleTagInserts
      )
      .on(
        "postgres_changes",
        { event: "DELETE", schema: "public", table: "tags" },
        handleTagDeletes
      )
      .on(
        "postgres_changes",
        { event: "UPDATE", schema: "public", table: "tags" },
        handleTagUpdates
      )
      .on(
        "postgres_changes",
        { event: "INSERT", schema: "public", table: "connectivity" },
        handleConnectivityInserts
      )
      .on(
        "postgres_changes",
        { event: "DELETE", schema: "public", table: "connectivity" },
        handleConnectivityDeletes
      )
      .on(
        "postgres_changes",
        { event: "UPDATE", schema: "public", table: "connectivity" },
        handleConnectivityUpdates
      )
      .on(
        "postgres_changes",
        { event: "INSERT", schema: "public", table: "sessions" },
        handleSessionInserts
      )
      .on(
        "postgres_changes",
        { event: "DELETE", schema: "public", table: "sessions" },
        handleSessionDeletes
      )
      .on(
        "postgres_changes",
        { event: "UPDATE", schema: "public", table: "sessions" },
        handleSessionUpdates
      )
      .subscribe((status: any) => {
        console.log("[DB Listener] Subscription status:", status);
        if (status === "SUBSCRIBED") {
          console.log(
            "[DB Listener] ✅ Successfully subscribed to real-time updates"
          );
          reconnectAttemptsRef.current = 0; // Reset reconnect attempts on successful connection
        } else if (status === "CHANNEL_ERROR") {
          console.warn(
            "[DB Listener] ❌ Channel error occurred. Reconnecting..."
          );
          handleReconnect();
        } else if (status === "TIMED_OUT") {
          console.warn(
            "[DB Listener] ⏰ Subscription timed out. Reconnecting..."
          );
          handleReconnect();
        } else if (status === "CLOSED") {
          console.log(
            `[DB Listener] 🔒 Channel closed: ${lastChannelIdRef.current}`
          );
          // Only reconnect if this isn't an immediate closure after subscription
          if (reconnectAttemptsRef.current > 0) {
            console.log("[DB Listener] Reconnecting...");
            handleReconnect();
          } else {
            console.log(
              "[DB Listener] Channel closed immediately after subscription, not reconnecting"
            );
          }
        }
      });

    return mainChannel;
  };

  // Handle reconnection with exponential backoff
  const handleReconnect = () => {
    if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
      console.error("[DB Listener] 🚫 Max reconnection attempts reached");
      return;
    }

    // Clear any existing timeout
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    const delay = Math.min(
      baseDelay * (reconnectAttemptsRef.current + 1),
      maxDelay
    );
    console.log(
      `[DB Listener] 🔄 Attempting reconnection in ${delay}ms (attempt ${
        reconnectAttemptsRef.current + 1
      }/${maxReconnectAttempts})`
    );

    reconnectTimeoutRef.current = setTimeout(() => {
      reconnectAttemptsRef.current++;
      cleanupChannels();
      const newChannel = setupChannel();
      if (newChannel) {
        channels.current.push(newChannel);
      }
    }, delay);
  };

  useEffect(() => {
    if (!scoutSupabase) {
      console.error("[DB Listener] No Supabase client available");
      return;
    }

    supabase.current = scoutSupabase;

    // Initial channel setup
    const mainChannel = setupChannel();
    if (mainChannel) {
      channels.current.push(mainChannel);
    }

    // Cleanup function
    return () => {
      console.log("[DB Listener] 🧹 Cleaning up channels");

      // Clear any pending reconnection attempts
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
        reconnectTimeoutRef.current = null;
      }

      // Reset reconnect attempts and channel tracking
      reconnectAttemptsRef.current = 0;
      lastChannelIdRef.current = null;

      // Clean up channels
      cleanupChannels();
    };
  }, [scoutSupabase, dispatch]);
}
