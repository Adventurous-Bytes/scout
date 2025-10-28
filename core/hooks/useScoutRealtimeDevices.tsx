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
    event: "INSERT" | "UPDATE" | "DELETE";
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    new?: IDevicePrettyLocation;
    old?: IDevicePrettyLocation;
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
      console.log("[Devices] Broadcast received:", payload.payload.event);

      const event = payload.payload.event;
      const data = payload.payload;

      switch (event) {
        case "INSERT":
          if (data.new) dispatch(addDevice(data.new));
          break;
        case "UPDATE":
          if (data.new) dispatch(updateDevice(data.new));
          break;
        case "DELETE":
          if (data.old) dispatch(deleteDevice(data.old));
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
          console.error(`[Devices] ❌ Failed to connect to herd ${herdId}`);
        }
      });
  };

  useEffect(() => {
    if (!scoutSupabase) return;

    cleanupChannels();

    // Create devices channel for active herd
    if (activeHerdId) {
      const channel = createDevicesChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [scoutSupabase, activeHerdId, handleDeviceBroadcast]);
}
