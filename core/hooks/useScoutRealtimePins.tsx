"use client";

import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IPin } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: IPin;
    old_record?: IPin;
  };
};

export function useScoutRealtimePins(
  scoutSupabase: SupabaseClient<Database>,
): [RealtimeData<IPin> | null, () => void] {
  const channels = useRef<RealtimeChannel[]>([]);
  const [latestPinUpdate, setLatestPinUpdate] =
    useState<RealtimeData<IPin> | null>(null);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  // Pin broadcast handler - just pass data, don't mutate state
  const handlePinBroadcast = useCallback((payload: BroadcastPayload) => {
    console.log("[Pins] Broadcast received:", payload.payload.operation);

    const data = payload.payload;
    const pinData = data.record || data.old_record;
    if (!pinData) return;

    let operation: EnumRealtimeOperation;
    switch (data.operation) {
      case "INSERT":
        operation = EnumRealtimeOperation.INSERT;
        console.log("[Pins] New pin received:", data.record);
        break;
      case "UPDATE":
        operation = EnumRealtimeOperation.UPDATE;
        console.log("[Pins] Pin updated:", data.record);
        break;
      case "DELETE":
        operation = EnumRealtimeOperation.DELETE;
        console.log("[Pins] Pin deleted:", data.old_record);
        break;
      default:
        return;
    }

    const realtimeData: RealtimeData<IPin> = {
      data: pinData,
      operation,
    };

    console.log(
      `[scout-core realtime] PIN ${data.operation} received for pin "${pinData.name}" (${pinData.id}):`,
      JSON.stringify(realtimeData),
    );

    setLatestPinUpdate(realtimeData);
  }, []);

  // Clear latest update
  const clearLatestUpdate = useCallback(() => {
    setLatestPinUpdate(null);
  }, []);

  const cleanupChannels = () => {
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];
  };

  const createPinsChannel = (herdId: string): RealtimeChannel => {
    return scoutSupabase
      .channel(`${herdId}-pins`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handlePinBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Pins] ✅ Connected to herd ${herdId} pins broadcasts`);
        } else if (status === "CHANNEL_ERROR") {
          console.warn(
            `[Pins] 🟡 Failed to connect to herd ${herdId} pins broadcasts`,
          );
        }
      });
  };

  useEffect(() => {
    cleanupChannels();

    // Clear previous update when switching herds
    clearLatestUpdate();

    // Create pins channel for active herd
    if (activeHerdId) {
      const channel = createPinsChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearLatestUpdate]);

  return [latestPinUpdate, clearLatestUpdate];
}
