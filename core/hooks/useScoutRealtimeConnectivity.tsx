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
import { getHoursAgoTimestamp } from "../helpers/time";

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
          console.log(
            `[CONNECTIVITY] INSERT for ${deviceId}, ${JSON.stringify(connectivityData)}`,
          );
          if (!updatedConnectivity[deviceId]) {
            updatedConnectivity[deviceId] = {
              most_recent: connectivityData,
              history: [],
            };
          } else {
            const newHistory = [
              updatedConnectivity[deviceId].most_recent,
              ...updatedConnectivity[deviceId].history,
            ].slice(0, 99);

            updatedConnectivity[deviceId] = {
              most_recent: connectivityData,
              history: newHistory,
            };
          }
          break;

        case "UPDATE":
          if (updatedConnectivity[deviceId]) {
            if (
              updatedConnectivity[deviceId].most_recent.id ===
              connectivityData.id
            ) {
              updatedConnectivity[deviceId] = {
                ...updatedConnectivity[deviceId],
                most_recent: connectivityData,
              };
            } else {
              const historyIndex = updatedConnectivity[
                deviceId
              ].history.findIndex((c) => c.id === connectivityData.id);
              if (historyIndex >= 0) {
                const newHistory = [...updatedConnectivity[deviceId].history];
                newHistory[historyIndex] = connectivityData;
                updatedConnectivity[deviceId] = {
                  ...updatedConnectivity[deviceId],
                  history: newHistory,
                };
              }
            }
          }
          break;

        case "DELETE":
          if (updatedConnectivity[deviceId]) {
            if (
              updatedConnectivity[deviceId].most_recent.id ===
              connectivityData.id
            ) {
              if (updatedConnectivity[deviceId].history.length === 0) {
                delete updatedConnectivity[deviceId];
              } else {
                updatedConnectivity[deviceId] = {
                  most_recent: updatedConnectivity[deviceId].history[0],
                  history: updatedConnectivity[deviceId].history.slice(1),
                };
              }
            } else {
              updatedConnectivity[deviceId] = {
                ...updatedConnectivity[deviceId],
                history: updatedConnectivity[deviceId].history.filter(
                  (c) => c.id !== connectivityData.id,
                ),
              };
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
    if (!gpsDeviceIds) return;

    const deviceIds = gpsDeviceIds.split(",").filter(Boolean).map(Number);

    if (deviceIds.length === 0) {
      return;
    }

    const timestampFilter = getHoursAgoTimestamp(1);
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
              const sortedData = trackerData
                .sort(
                  (a, b) =>
                    new Date(b.timestamp_start || 0).getTime() -
                    new Date(a.timestamp_start || 0).getTime(),
                )
                .slice(0, 100);

              connectivityData[deviceId] = {
                most_recent: sortedData[0],
                history: sortedData.slice(1), // Exclude the most recent item
              };
            }
          }
        } catch (error) {
          // Silent error handling
        }
      }),
    );

    dispatch(setActiveHerdGpsTrackersConnectivity(connectivityData));
  }, [gpsDeviceIds, dispatch]);

  useEffect(() => {
    if (!scoutSupabase || gpsDeviceIds === "") return;

    // Clean up existing channels
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];

    // Create connectivity channel
    const channel = scoutSupabase
      .channel(`${activeHerdId}-connectivity`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleConnectivityBroadcast)
      .subscribe();

    channels.current.push(channel);

    // Fetch initial data
    fetchInitialData();

    return () => {
      channels.current.forEach((ch) => scoutSupabase.removeChannel(ch));
      channels.current = [];
    };
  }, [scoutSupabase, gpsDeviceIds, activeHerdId, handleConnectivityBroadcast]);
}
