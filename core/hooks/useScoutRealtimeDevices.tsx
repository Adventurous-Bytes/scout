"use client";

import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IDevicePrettyLocation } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: IDevicePrettyLocation;
    old_record?: IDevicePrettyLocation;
  };
};

export function useScoutRealtimeDevices(
  scoutSupabase: SupabaseClient<Database>,
): [RealtimeData<IDevicePrettyLocation> | null, () => void] {
  const channels = useRef<RealtimeChannel[]>([]);
  const [latestDeviceUpdate, setLatestDeviceUpdate] =
    useState<RealtimeData<IDevicePrettyLocation> | null>(null);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  // Device broadcast handler - just pass data, don't mutate state
  const handleDeviceBroadcast = useCallback((payload: BroadcastPayload) => {
    console.log("[Devices] Broadcast received:", payload.payload.operation);

    const data = payload.payload;
    const deviceData = data.record || data.old_record;
    if (!deviceData) return;

    let operation: EnumRealtimeOperation;
    switch (data.operation) {
      case "INSERT":
        operation = EnumRealtimeOperation.INSERT;
        console.log("[Devices] New device received:", data.record);
        break;
      case "UPDATE":
        operation = EnumRealtimeOperation.UPDATE;
        console.log("[Devices] Device updated:", data.record);
        break;
      case "DELETE":
        operation = EnumRealtimeOperation.DELETE;
        console.log("[Devices] Device deleted:", data.old_record);
        break;
      default:
        return;
    }

    const realtimeData: RealtimeData<IDevicePrettyLocation> = {
      data: deviceData,
      operation,
    };

    console.log(
      `[scout-core realtime] DEVICE ${data.operation} received:`,
      JSON.stringify(realtimeData),
    );

    setLatestDeviceUpdate(realtimeData);
  }, []);

  // Clear latest update
  const clearLatestUpdate = useCallback(() => {
    setLatestDeviceUpdate(null);
  }, []);

  const cleanupChannels = () => {
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];
  };

  const createDevicesChannel = (herdId: string): RealtimeChannel => {
    return scoutSupabase
      .channel(`${herdId}-devices`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleDeviceBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Devices] ✅ Connected to herd ${herdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.warn(`[Devices] 🟡 Failed to connect to herd ${herdId}`);
        }
      });
  };

  useEffect(() => {
    cleanupChannels();

    // Clear previous update when switching herds
    clearLatestUpdate();

    // Create devices channel for active herd
    if (activeHerdId) {
      const channel = createDevicesChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearLatestUpdate]);

  return [latestDeviceUpdate, clearLatestUpdate];
}
