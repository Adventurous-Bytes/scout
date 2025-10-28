"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import { setActiveHerdGpsTrackersConnectivity } from "../store/scout";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IConnectivityWithCoordinates, DeviceType } from "../types/db";
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
  const [isLoading, setIsLoading] = useState(false);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );
  const activeHerdGpsTrackersConnectivity = useSelector(
    (state: RootState) => state.scout.active_herd_gps_trackers_connectivity,
  );
  const herdModules = useSelector(
    (state: RootState) => state.scout.herd_modules,
  );

  // Connectivity broadcast handler
  const handleConnectivityBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      console.log("[Connectivity] Broadcast received:", payload.payload.event);

      const event = payload.payload.event;
      const data = payload.payload;
      const connectivityData = data.new || data.old;

      // Only process tracker connectivity data (no session_id)
      if (!connectivityData || connectivityData.session_id) {
        return;
      }

      const deviceId = connectivityData.device_id;
      if (!deviceId) return;

      const currentConnectivity: MapDeviceIdToConnectivity = {
        ...activeHerdGpsTrackersConnectivity,
      };
      console.log("[Connectivity] Current connectivity:", currentConnectivity);
      switch (event) {
        case "INSERT":
          if (!currentConnectivity[deviceId]) {
            currentConnectivity[deviceId] = [];
          }
          currentConnectivity[deviceId].push(connectivityData);

          // Keep only recent 100 entries
          if (currentConnectivity[deviceId].length > 100) {
            currentConnectivity[deviceId] = currentConnectivity[deviceId]
              .sort(
                (a, b) =>
                  new Date(b.timestamp_start || 0).getTime() -
                  new Date(a.timestamp_start || 0).getTime(),
              )
              .slice(0, 100);
          }
          break;

        case "UPDATE":
          if (currentConnectivity[deviceId]) {
            const index = currentConnectivity[deviceId].findIndex(
              (c) => c.id === connectivityData.id,
            );
            if (index >= 0) {
              currentConnectivity[deviceId][index] = connectivityData;
            }
          }
          break;

        case "DELETE":
          if (currentConnectivity[deviceId]) {
            currentConnectivity[deviceId] = currentConnectivity[
              deviceId
            ].filter((c) => c.id !== connectivityData.id);

            if (currentConnectivity[deviceId].length === 0) {
              delete currentConnectivity[deviceId];
            }
          }
          break;
      }

      dispatch(setActiveHerdGpsTrackersConnectivity(currentConnectivity));
    },
    [activeHerdGpsTrackersConnectivity, dispatch],
  );

  const cleanupChannels = () => {
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];
  };

  const createConnectivityChannel = (herdId: string): RealtimeChannel => {
    return scoutSupabase
      .channel(`${herdId}-connectivity`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleConnectivityBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Connectivity] ✅ Connected to herd ${herdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.error(
            `[Connectivity] ❌ Failed to connect to herd ${herdId}`,
          );
        }
      });
  };

  // Fetch initial connectivity data for GPS trackers
  const fetchInitialConnectivityData = useCallback(async () => {
    if (!activeHerdId || isLoading) return;

    // Find the active herd module
    const activeHerdModule = herdModules.find(
      (hm) => hm.herd.id.toString() === activeHerdId,
    );
    if (!activeHerdModule) return;

    // Get GPS tracker devices from the herd
    const gpsTrackerDevices = activeHerdModule.devices.filter(
      (device) =>
        device.device_type === "gps_tracker" ||
        device.device_type === "gps_tracker_vehicle" ||
        device.device_type === "gps_tracker_person",
    );

    if (gpsTrackerDevices.length === 0) {
      console.log("[Connectivity] No GPS trackers found in herd");
      return;
    }

    setIsLoading(true);
    console.log(
      `[Connectivity] Fetching last day connectivity for ${gpsTrackerDevices.length} GPS trackers`,
    );

    // Calculate timestamp for last 24 hours
    const timestampFilter = getDaysAgoTimestamp(1);

    const connectivityData: MapDeviceIdToConnectivity = {};

    try {
      // Fetch connectivity for each GPS tracker
      await Promise.all(
        gpsTrackerDevices.map(async (device) => {
          try {
            if (!device.id) return;
            const response = await server_get_connectivity_by_device_id(
              device.id,
              timestampFilter,
            );

            if (response.status === EnumWebResponse.SUCCESS && response.data) {
              // Filter out any data with session_id (only tracker data)
              const trackerConnectivity = response.data.filter(
                (conn) => !conn.session_id,
              );

              if (trackerConnectivity.length > 0 && device.id) {
                // Keep only most recent 100 entries per device
                connectivityData[device.id] = trackerConnectivity
                  .sort(
                    (a, b) =>
                      new Date(b.timestamp_start || 0).getTime() -
                      new Date(a.timestamp_start || 0).getTime(),
                  )
                  .slice(0, 100);

                console.log(
                  `[Connectivity] Loaded ${connectivityData[device.id]?.length} records for device ${device.id}`,
                );
              }
            }
          } catch (error) {
            console.warn(
              `[Connectivity] Failed to fetch data for device ${device.id}:`,
              error,
            );
          }
        }),
      );

      // Update the store with initial connectivity data
      dispatch(setActiveHerdGpsTrackersConnectivity(connectivityData));
      console.log(
        `[Connectivity] Initial data loaded for ${Object.keys(connectivityData).length} devices`,
      );
    } catch (error) {
      console.error("[Connectivity] Error fetching initial data:", error);
    } finally {
      setIsLoading(false);
    }
  }, [activeHerdId, herdModules, isLoading, dispatch]);

  useEffect(() => {
    if (!scoutSupabase) return;

    cleanupChannels();

    // Create connectivity channel for active herd
    if (activeHerdId) {
      const channel = createConnectivityChannel(activeHerdId);
      channels.current.push(channel);

      // Fetch initial connectivity data
      fetchInitialConnectivityData();
    }

    return cleanupChannels;
  }, [
    scoutSupabase,
    activeHerdId,
    handleConnectivityBroadcast,
    fetchInitialConnectivityData,
  ]);
}
