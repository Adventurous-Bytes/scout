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
  const dispatch = useAppDispatch();
  const [latestPartUpdate, setLatestPartUpdate] =
    useState<RealtimeData<IPart> | null>(null);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  const herdModules = useSelector(
    (state: RootState) => state.scout.herd_modules,
  );

  // Part broadcast handler
  const handlePartBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      console.log("[Parts] Broadcast received:", payload.payload.operation);

      const data = payload.payload;
      const partData = data.record || data.old_record;
      if (!partData) return;

      let operation: EnumRealtimeOperation;

      // Find the target herd module and device
      const herdModule = herdModules.find(
        (hm) => hm.herd.id.toString() === activeHerdId,
      );

      if (!herdModule) {
        console.warn("[Parts] No herd module found for active herd");
        return;
      }

      const targetDevice = herdModule.devices.find(
        (device) => device.id === partData.device_id,
      );

      if (!targetDevice) {
        console.warn(`[Parts] No device found with ID: ${partData.device_id}`);
        return;
      }

      // Ensure device has parts array
      if (!targetDevice.parts) {
        targetDevice.parts = [];
      }

      switch (data.operation) {
        case "INSERT":
          operation = EnumRealtimeOperation.INSERT;
          if (data.record) {
            console.log("[Parts] New part received:", data.record);
            // Add part to device's parts array if not already present
            const existingPartIndex = targetDevice.parts.findIndex(
              (p) => p.id === data.record!.id,
            );
            if (existingPartIndex === -1) {
              targetDevice.parts.push(data.record);
            }
          }
          break;
        case "UPDATE":
          operation = EnumRealtimeOperation.UPDATE;
          if (data.record) {
            console.log("[Parts] Part updated:", data.record);
            // Update existing part in device's parts array
            const partIndex = targetDevice.parts.findIndex(
              (p) => p.id === data.record!.id,
            );
            if (partIndex !== -1) {
              targetDevice.parts[partIndex] = data.record;
            } else {
              // Part not found, add it
              targetDevice.parts.push(data.record);
            }
          }
          break;
        case "DELETE":
          operation = EnumRealtimeOperation.DELETE;
          if (data.old_record) {
            console.log("[Parts] Part deleted:", data.old_record);
            // Remove part from device's parts array
            targetDevice.parts = targetDevice.parts.filter(
              (p) => p.id !== data.old_record!.id,
            );
          }
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
    },
    [dispatch, activeHerdId, herdModules],
  );

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
