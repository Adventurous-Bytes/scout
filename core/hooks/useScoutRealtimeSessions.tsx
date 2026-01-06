"use client";

import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { ISessionWithCoordinates } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: ISessionWithCoordinates;
    old_record?: ISessionWithCoordinates;
  };
};

export function useScoutRealtimeSessions(
  scoutSupabase: SupabaseClient<Database>,
): [RealtimeData<ISessionWithCoordinates> | null, () => void] {
  const channels = useRef<RealtimeChannel[]>([]);
  const [latestSessionUpdate, setLatestSessionUpdate] =
    useState<RealtimeData<ISessionWithCoordinates> | null>(null);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  // Session broadcast handler - just pass data, don't mutate state
  const handleSessionBroadcast = useCallback((payload: BroadcastPayload) => {
    console.log("[Sessions] Broadcast received:", payload.payload.operation);

    const data = payload.payload;
    const sessionData = data.record || data.old_record;
    if (!sessionData) return;

    let operation: EnumRealtimeOperation;
    switch (data.operation) {
      case "INSERT":
        operation = EnumRealtimeOperation.INSERT;
        console.log("[Sessions] New session received:", data.record);
        break;
      case "UPDATE":
        operation = EnumRealtimeOperation.UPDATE;
        console.log("[Sessions] Session updated:", data.record);
        break;
      case "DELETE":
        operation = EnumRealtimeOperation.DELETE;
        console.log("[Sessions] Session deleted:", data.old_record);
        break;
      default:
        return;
    }

    const realtimeData: RealtimeData<ISessionWithCoordinates> = {
      data: sessionData,
      operation,
    };

    console.log(
      `[scout-core realtime] SESSION ${data.operation} received:`,
      JSON.stringify(realtimeData),
    );

    setLatestSessionUpdate(realtimeData);
  }, []);

  // Clear latest update
  const clearLatestUpdate = useCallback(() => {
    setLatestSessionUpdate(null);
  }, []);

  const cleanupChannels = () => {
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];
  };

  const createSessionsChannel = (herdId: string): RealtimeChannel => {
    return scoutSupabase
      .channel(`${herdId}-sessions`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleSessionBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Sessions] ✅ Connected to herd ${herdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.warn(`[Sessions] 🟡 Failed to connect to herd ${herdId}`);
        }
      });
  };

  useEffect(() => {
    cleanupChannels();

    // Clear previous update when switching herds
    clearLatestUpdate();

    // Create sessions channel for active herd
    if (activeHerdId) {
      const channel = createSessionsChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearLatestUpdate]);

  return [latestSessionUpdate, clearLatestUpdate];
}
