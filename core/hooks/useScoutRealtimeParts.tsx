"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IPart } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: IPart & { herd_id: number };
    old_record?: IPart & { herd_id: number };
  };
};

export function useScoutRealtimeParts(
  scoutSupabase: SupabaseClient<Database>,
): [RealtimeData<IPart> | null, () => void] {
  const channels = useRef<RealtimeChannel[]>([]);
  const [latestPartUpdate, setLatestPartUpdate] =
    useState<RealtimeData<IPart> | null>(null);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  // Part broadcast handler - just pass data, don't mutate state
  const handlePartBroadcast = useCallback((payload: BroadcastPayload) => {
    console.log("[Parts] Broadcast received:", payload.payload.operation);

    const data = payload.payload;
    const partData = data.record || data.old_record;
    if (!partData) return;

    let operation: EnumRealtimeOperation;

    switch (data.operation) {
      case "INSERT":
        operation = EnumRealtimeOperation.INSERT;
        console.log("[Parts] New part received:", data.record);
        break;
      case "UPDATE":
        operation = EnumRealtimeOperation.UPDATE;
        console.log("[Parts] Part updated:", data.record);
        break;
      case "DELETE":
        operation = EnumRealtimeOperation.DELETE;
        console.log("[Parts] Part deleted:", data.old_record);
        break;
      default:
        return;
    }

    const realtimeData: RealtimeData<IPart> = {
      data: partData,
      operation,
    };

    console.log(
      `[scout-core realtime] PART ${data.operation} received:`,
      JSON.stringify(realtimeData),
    );

    setLatestPartUpdate(realtimeData);
  }, []);

  // Clear latest update
  const clearLatestUpdate = useCallback(() => {
    setLatestPartUpdate(null);
  }, []);

  const cleanupChannels = () => {
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];
  };

  const createPartsChannel = (herdId: string): RealtimeChannel => {
    return scoutSupabase
      .channel(`${herdId}-parts`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handlePartBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Parts] ✅ Connected to herd ${herdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.warn(`[Parts] 🟡 Failed to connect to herd ${herdId}`);
        }
      });
  };

  useEffect(() => {
    cleanupChannels();

    // Clear previous update when switching herds
    clearLatestUpdate();

    // Create parts channel for active herd
    if (activeHerdId) {
      const channel = createPartsChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearLatestUpdate]);

  return [latestPartUpdate, clearLatestUpdate];
}
