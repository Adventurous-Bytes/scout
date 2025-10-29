"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback } from "react";
import { addDevice, deleteDevice, updateDevice } from "../store/scout";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IDevicePrettyLocation } from "../types/db";
import { RootState } from "../store/scout";

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
) {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  // Device broadcast handler
  const handleDeviceBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      console.log("[Devices] Broadcast received:", payload.payload.operation);

      const data = payload.payload;

      switch (data.operation) {
        case "INSERT":
          if (data.record) dispatch(addDevice(data.record));
          break;
        case "UPDATE":
          if (data.record) dispatch(updateDevice(data.record));
          break;
        case "DELETE":
          if (data.old_record) dispatch(deleteDevice(data.old_record));
          break;
      }
    },
    [dispatch],
  );

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

    // Create devices channel for active herd
    if (activeHerdId) {
      const channel = createDevicesChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId]);
}
