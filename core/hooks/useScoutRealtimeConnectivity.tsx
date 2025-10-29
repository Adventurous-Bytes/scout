"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState, useMemo } from "react";
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
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: IConnectivityWithCoordinates;
    old_record?: IConnectivityWithCoordinates;
  };
};

export function useScoutRealtimeConnectivity(
  scoutSupabase: SupabaseClient<Database>,
) {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );
  const connectivity = useSelector(
    (state: RootState) => state.scout.active_herd_gps_trackers_connectivity,
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

      const deviceId = connectivityData.device_id;
      const updatedConnectivity = { ...connectivity };

      switch (data.operation) {
        case "INSERT":
          if (!updatedConnectivity[deviceId]) {
            updatedConnectivity[deviceId] = [];
          } else {
            // Create a copy of the existing array to avoid mutating immutable state
            updatedConnectivity[deviceId] = [...updatedConnectivity[deviceId]];
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
            // Create a copy of the array before modifying
            updatedConnectivity[deviceId] = [...updatedConnectivity[deviceId]];

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
            // Filter creates a new array, so this is safe
            updatedConnectivity[deviceId] = updatedConnectivity[
              deviceId
            ].filter((c) => c.id !== connectivityData.id);

            if (updatedConnectivity[deviceId].length === 0) {
              delete updatedConnectivity[deviceId];
            }
          }
          break;
      }
      console.log(
        "[Connectivity] updating tracker connectivity in response to broadcast",
      );
      dispatch(setActiveHerdGpsTrackersConnectivity(updatedConnectivity));
    },
    [connectivity, dispatch],
  );

  // Fetch initial connectivity data
  const fetchInitialData = useCallback(async () => {
    if (!activeHerdId || !gpsDeviceIds) return;

    const deviceIds = gpsDeviceIds.split(",").filter(Boolean).map(Number);

    if (deviceIds.length === 0) {
      return;
    }

    console.log(
      `[Connectivity] Loading data for ${deviceIds.length} GPS trackers`,
    );

    const timestampFilter = getDaysAgoTimestamp(1);
    const connectivityData: MapDeviceIdToConnectivity = {};

    await Promise.all(
      deviceIds.map(async (deviceId) => {
        try {
          const response = await server_get_connectivity_by_device_id(
            deviceId,
            timestampFilter,
          );

          if (response.status === EnumWebResponse.SUCCESS && response.data) {
            const trackerData = response.data.filter(
              (conn) => !conn.session_id,
            );
            if (trackerData.length > 0) {
              connectivityData[deviceId] = trackerData
                .sort(
                  (a, b) =>
                    new Date(b.timestamp_start || 0).getTime() -
                    new Date(a.timestamp_start || 0).getTime(),
                )
                .slice(0, 100);
            }
          } else {
            console.warn(
              `[Connectivity] API error for device ${deviceId}:`,
              response.msg || "Unknown error",
            );
          }
        } catch (error) {
          console.warn(
            `[Connectivity] Failed to fetch data for device ${deviceId}:`,
            error,
          );
        }
      }),
    );

    dispatch(setActiveHerdGpsTrackersConnectivity(connectivityData));

    console.log(
      `[Connectivity] Loaded data for ${Object.keys(connectivityData).length} devices`,
    );
  }, [activeHerdId, gpsDeviceIds, dispatch]);

  useEffect(() => {
    if (!scoutSupabase || !gpsDeviceIds) return;

    // Clean up existing channels
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];

    // Create connectivity channel
    const channel = scoutSupabase
      .channel(`${activeHerdId}-connectivity`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleConnectivityBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Connectivity] ✅ Connected to herd ${activeHerdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.warn(
            `[Connectivity] 🟡 Failed to connect to herd ${activeHerdId}`,
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
  }, [gpsDeviceIds]);
}
