"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";

import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IEventAndTagsPrettyLocation } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";
import { scoutApi } from "../store/api";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: IEventAndTagsPrettyLocation;
    old_record?: IEventAndTagsPrettyLocation;
  };
};

export function useScoutRealtimeEvents(
  scoutSupabase: SupabaseClient<Database>,
  invalidateRTKQuery: boolean = true,
): [RealtimeData<IEventAndTagsPrettyLocation> | null, () => void] {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();
  const [latestEventUpdate, setLatestEventUpdate] =
    useState<RealtimeData<IEventAndTagsPrettyLocation> | null>(null);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  // Event broadcast handler
  const handleEventBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      console.log("[Events] Broadcast received:", payload.payload.operation);

      const data = payload.payload;

      const eventData = data.record || data.old_record;
      if (!eventData) return;

      let operation: EnumRealtimeOperation;
      switch (data.operation) {
        case "INSERT":
          operation = EnumRealtimeOperation.INSERT;
          if (data.record && invalidateRTKQuery) {
            console.log(
              "[Events] New event received, invalidating RTK Query cache:",
              data.record,
            );
            // Invalidate all events queries to refetch fresh data
            dispatch(scoutApi.util.invalidateTags(["Event"]));
          }
          break;
        case "UPDATE":
          operation = EnumRealtimeOperation.UPDATE;
          if (data.record && invalidateRTKQuery) {
            console.log(
              "[Events] Event updated, invalidating RTK Query cache:",
              data.record,
            );
            // Invalidate specific event and list queries
            dispatch(
              scoutApi.util.invalidateTags([
                { type: "Event", id: data.record.id || "unknown" },
                { type: "Event", id: "LIST" },
              ]),
            );
          }
          break;
        case "DELETE":
          operation = EnumRealtimeOperation.DELETE;
          if (data.old_record && invalidateRTKQuery) {
            console.log(
              "[Events] Event deleted, invalidating RTK Query cache:",
              data.old_record,
            );
            // Invalidate all events queries since item was deleted
            dispatch(scoutApi.util.invalidateTags(["Event"]));
          }
          break;
        default:
          return;
      }

      const realtimeData: RealtimeData<IEventAndTagsPrettyLocation> = {
        data: eventData,
        operation,
      };

      console.log(
        `[scout-core realtime] EVENT ${data.operation} received:`,
        JSON.stringify(realtimeData),
      );

      setLatestEventUpdate(realtimeData);
    },
    [dispatch, invalidateRTKQuery],
  );

  // Clear latest update
  const clearLatestUpdate = useCallback(() => {
    setLatestEventUpdate(null);
  }, []);

  const cleanupChannels = () => {
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];
  };

  const createEventsChannel = (herdId: string): RealtimeChannel => {
    return scoutSupabase
      .channel(`${herdId}-events`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleEventBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Events] ✅ Connected to herd ${herdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.warn(`[Events] 🟡 Failed to connect to herd ${herdId}`);
        }
      });
  };

  useEffect(() => {
    cleanupChannels();

    // Clear previous update when switching herds
    clearLatestUpdate();

    // Create events channel for active herd
    if (activeHerdId) {
      const channel = createEventsChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearLatestUpdate]);

  return [latestEventUpdate, clearLatestUpdate];
}
