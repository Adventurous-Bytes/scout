"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import { setActiveHerdGpsTrackersConnectivity } from "../store/scout";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IConnectivityWithCoordinates } from "../types/db";
import { MapDeviceIdToConnectivity } from "../types/connectivity";
import { RootState } from "../store/scout";
import { server_get_connectivity_by_device_id } from "../helpers/connectivity";
import { EnumWebResponse } from "../types/requests";
import { getDaysAgoTimestamp } from "../helpers/time";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    event: "INSERT" | "UPDATE" | "DELETE";
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    new?: IConnectivityWithCoordinates;
    old?: IConnectivityWithCoordinates;
  };
};

export function useScoutRealtimeConnectivity(
  scoutSupabase: SupabaseClient<Database>,
) {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();
  const [hasInitialized, setHasInitialized] = useState<string | null>(null);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );
  const connectivity = useSelector(
    (state: RootState) => state.scout.active_herd_gps_trackers_connectivity,
  );
  const herdModules = useSelector(
    (state: RootState) => state.scout.herd_modules,
  );

  // Handle connectivity broadcasts
  const handleConnectivityBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      const { event, payload: data } = payload;
      const connectivityData = data.new || data.old;

      // Only process GPS tracker data (no session_id)
      if (!connectivityData?.device_id || connectivityData.session_id) {
        return;
      }

      const deviceId = connectivityData.device_id;
      const updatedConnectivity = { ...connectivity };

      switch (event) {
        case "INSERT":
          if (!updatedConnectivity[deviceId]) {
            updatedConnectivity[deviceId] = [];
          }
          updatedConnectivity[deviceId].push(connectivityData);

          // Keep only recent 100 entries
          if (updatedConnectivity[deviceId].length > 100) {
            updatedConnectivity[deviceId] = updatedConnectivity[deviceId]
              .sort(
                (a, b) =>
                  new Date(b.timestamp_start || 0).getTime() -
                  new Date(a.timestamp_start || 0).getTime(),
              )
              .slice(0, 100);
          }
          break;

        case "UPDATE":
          if (updatedConnectivity[deviceId]) {
            const index = updatedConnectivity[deviceId].findIndex(
              (c) => c.id === connectivityData.id,
            );
            if (index >= 0) {
              updatedConnectivity[deviceId][index] = connectivityData;
            }
          }
          break;

        case "DELETE":
          if (updatedConnectivity[deviceId]) {
            updatedConnectivity[deviceId] = updatedConnectivity[
              deviceId
            ].filter((c) => c.id !== connectivityData.id);
            if (updatedConnectivity[deviceId].length === 0) {
              delete updatedConnectivity[deviceId];
            }
          }
          break;
      }

      dispatch(setActiveHerdGpsTrackersConnectivity(updatedConnectivity));
    },
    [connectivity, dispatch],
  );

  // Fetch initial connectivity data
  const fetchInitialData = useCallback(async () => {
    if (!activeHerdId || hasInitialized === activeHerdId) return;

    const herdId = activeHerdId; // Type narrowing
    const activeHerdModule = herdModules.find(
      (hm) => hm.herd.id.toString() === herdId,
    );
    if (!activeHerdModule) return;

    const gpsDevices = activeHerdModule.devices.filter(
      (device) =>
        device.device_type &&
        ["gps_tracker", "gps_tracker_vehicle", "gps_tracker_person"].includes(
          device.device_type,
        ),
    );

    if (gpsDevices.length === 0) {
      setHasInitialized(herdId);
      return;
    }

    console.log(
      `[Connectivity] Loading data for ${gpsDevices.length} GPS trackers`,
    );

    const timestampFilter = getDaysAgoTimestamp(1);
    const connectivityData: MapDeviceIdToConnectivity = {};

    await Promise.all(
      gpsDevices.map(async (device) => {
        if (!device.id) return;

        try {
          const response = await server_get_connectivity_by_device_id(
            device.id,
            timestampFilter,
          );

          if (response.status === EnumWebResponse.SUCCESS && response.data) {
            const trackerData = response.data.filter(
              (conn) => !conn.session_id,
            );
            if (trackerData.length > 0) {
              connectivityData[device.id] = trackerData
                .sort(
                  (a, b) =>
                    new Date(b.timestamp_start || 0).getTime() -
                    new Date(a.timestamp_start || 0).getTime(),
                )
                .slice(0, 100);
            }
          } else {
            console.error(
              `[Connectivity] API error for device ${device.id}:`,
              response.msg || "Unknown error",
            );
          }
        } catch (error) {
          console.error(
            `[Connectivity] Failed to fetch data for device ${device.id}:`,
            error,
          );
        }
      }),
    );

    dispatch(setActiveHerdGpsTrackersConnectivity(connectivityData));
    setHasInitialized(herdId);

    console.log(
      `[Connectivity] Loaded data for ${Object.keys(connectivityData).length} devices`,
    );
  }, [activeHerdId, herdModules, hasInitialized, dispatch]);

  useEffect(() => {
    if (!scoutSupabase || !activeHerdId) return;

    // Clean up existing channels
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];

    // Reset initialization when herd changes
    setHasInitialized(null);

    // Create connectivity channel
    const channel = scoutSupabase
      .channel(`${activeHerdId}-connectivity`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleConnectivityBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Connectivity] ✅ Connected to herd ${activeHerdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.error(
            `[Connectivity] ❌ Failed to connect to herd ${activeHerdId}`,
          );
        }
      });

    channels.current.push(channel);

    // Fetch initial data
    fetchInitialData();

    return () => {
      channels.current.forEach((ch) => scoutSupabase.removeChannel(ch));
      channels.current = [];
    };
  }, [
    scoutSupabase,
    activeHerdId,
    handleConnectivityBroadcast,
    fetchInitialData,
  ]);
}
