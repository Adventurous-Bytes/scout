"use client";

import { useAppDispatch } from "../store/hooks";
import { useEffect, useRef } from "react";
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
import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";

export function useScoutDbListener(scoutSupabase: SupabaseClient<Database>) {
  const supabase = useRef<any>(null);
  const channels = useRef<any[]>([]);
  const dispatch = useAppDispatch();

  function handleTagInserts(payload: any) {
    console.log("[DB Listener] Tag INSERT received:", payload.new);
    dispatch(addTag(payload.new));
  }

  function handleTagDeletes(payload: any) {
    console.log("[DB Listener] Tag DELETE received:", payload.old);
    console.log("[DB Listener] Tag DELETE - payload structure:", {
      hasOld: !!payload.old,
      oldId: payload.old?.id,
      oldEventId: payload.old?.event_id,
      oldClassName: payload.old?.class_name,
      fullPayload: payload,
    });

    if (!payload.old || !payload.old.id) {
      console.error(
        "[DB Listener] Tag DELETE - Invalid payload, missing tag data"
      );
      return;
    }

    console.log(
      "[DB Listener] Tag DELETE - Dispatching deleteTag action with ID:",
      payload.old.id
    );
    dispatch(deleteTag(payload.old));
  }

  function handleTagUpdates(payload: any) {
    console.log("[DB Listener] Tag UPDATE received:", payload.new);
    dispatch(updateTag(payload.new));
  }

  async function handleDeviceInserts(payload: any) {
    console.log("[DB Listener] Device INSERT received:", payload.new);
    // For now, just dispatch the raw payload since we don't have the device helper
    dispatch(addDevice(payload.new));
  }

  function handleDeviceDeletes(payload: any) {
    console.log("[DB Listener] Device DELETE received:", payload.old);
    dispatch(deleteDevice(payload.old));
  }

  async function handleDeviceUpdates(payload: any) {
    console.log("[DB Listener] Device UPDATE received:", payload.new);
    // For now, just dispatch the raw payload since we don't have the device helper
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
        console.error("[DB Listener] Auth test failed:", err);
      }
    };
    testAuth();

    // Create a single channel for all operations with unique name
    const channelName = `scout_realtime_${Date.now()}`;
    console.log("[DB Listener] Creating channel:", channelName);
    const mainChannel = scoutSupabase.channel(channelName);

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
      .subscribe((status: any) => {
        console.log("[DB Listener] Subscription status:", status);
        if (status === "SUBSCRIBED") {
          console.log(
            "[DB Listener] ✅ Successfully subscribed to real-time updates"
          );
        } else if (status === "CHANNEL_ERROR") {
          console.error("[DB Listener] ❌ Channel error occurred");
        } else if (status === "TIMED_OUT") {
          console.error("[DB Listener] ⏰ Subscription timed out");
        } else if (status === "CLOSED") {
          console.log("[DB Listener] 🔒 Channel closed");
        }
      });

    channels.current.push(mainChannel);

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
        console.error("[DB Listener] ❌ System error:", error);
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
