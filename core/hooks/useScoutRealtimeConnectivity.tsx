"use client";

import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState, useMemo } from "react";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IConnectivityWithCoordinates } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: IConnectivityWithCoordinates;
    old_record?: IConnectivityWithCoordinates;
  };
};

export function useScoutRealtimeConnectivity(
  scoutSupabase: SupabaseClient<Database>,
): RealtimeData<IConnectivityWithCoordinates>[] {
  const channels = useRef<RealtimeChannel[]>([]);
  const [newConnectivityItems, setNewConnectivityItems] = useState<
    RealtimeData<IConnectivityWithCoordinates>[]
  >([]);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );
  const herdModules = useSelector(
    (state: RootState) => state.scout.herd_modules,
  );

  // Create stable reference for GPS device IDs to prevent unnecessary refetching
  const gpsDeviceIds = useMemo(() => {
    if (!activeHerdId) return "";

    const activeHerdModule = herdModules.find(
      (hm) => hm.herd.id.toString() === activeHerdId,
    );
    if (!activeHerdModule) return "";

    const gpsDevices = activeHerdModule.devices.filter(
      (device) =>
        device.device_type &&
        ["gps_tracker", "gps_tracker_vehicle", "gps_tracker_person"].includes(
          device.device_type,
        ),
    );

    return gpsDevices
      .map((d) => d.id)
      .filter(Boolean)
      .sort()
      .join(",");
  }, [activeHerdId, herdModules]);

  // Handle connectivity broadcasts
  const handleConnectivityBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      const { event, payload: data } = payload;
      const connectivityData = data.record || data.old_record;

      // Only process GPS tracker data (no session_id)
      if (!connectivityData?.device_id || connectivityData.session_id) {
        return;
      }

      let operation: EnumRealtimeOperation;
      switch (data.operation) {
        case "INSERT":
          operation = EnumRealtimeOperation.INSERT;
          break;
        case "UPDATE":
          operation = EnumRealtimeOperation.UPDATE;
          break;
        case "DELETE":
          operation = EnumRealtimeOperation.DELETE;
          break;
        default:
          return;
      }

      console.log(
        `[scout-core realtime] CONNECTIVITY ${data.operation} received for device ${connectivityData.device_id}:`,
        JSON.stringify(connectivityData),
      );

      const realtimeData: RealtimeData<IConnectivityWithCoordinates> = {
        data: connectivityData,
        operation,
      };

      setNewConnectivityItems((prev) => [realtimeData, ...prev]);
    },
    [],
  );

  // Clear new items when gps device IDs change (herd change)
  const clearNewItems = useCallback(() => {
    setNewConnectivityItems([]);
  }, []);

  useEffect(() => {
    if (!scoutSupabase || gpsDeviceIds === "") return;

    // Clean up existing channels
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];

    // Clear previous items when switching herds
    clearNewItems();

    // Create connectivity channel
    const channel = scoutSupabase
      .channel(`${activeHerdId}-connectivity`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleConnectivityBroadcast)
      .subscribe();

    channels.current.push(channel);

    return () => {
      channels.current.forEach((ch) => scoutSupabase.removeChannel(ch));
      channels.current = [];
    };
  }, [
    scoutSupabase,
    gpsDeviceIds,
    activeHerdId,
    handleConnectivityBroadcast,
    clearNewItems,
  ]);

  return newConnectivityItems;
}
