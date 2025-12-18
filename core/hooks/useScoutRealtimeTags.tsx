"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";

import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { ITagPrettyLocation } from "../types/db";
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
    record?: ITagPrettyLocation;
    old_record?: ITagPrettyLocation;
  };
};

export function useScoutRealtimeTags(
  scoutSupabase: SupabaseClient<Database>,
  invalidateRTKQuery: boolean = true,
): [RealtimeData<ITagPrettyLocation> | null, () => void] {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();
  const [latestTagUpdate, setLatestTagUpdate] =
    useState<RealtimeData<ITagPrettyLocation> | null>(null);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  // Tag broadcast handler
  const handleTagBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      console.log("[Tags] Broadcast received:", payload.payload.operation);

      const data = payload.payload;

      const tagData = data.record || data.old_record;
      if (!tagData) return;

      let operation: EnumRealtimeOperation;
      switch (data.operation) {
        case "INSERT":
          operation = EnumRealtimeOperation.INSERT;
          if (data.record && invalidateRTKQuery) {
            console.log(
              "[Tags] New tag received, invalidating RTK Query cache:",
              data.record,
            );
            // Tags are part of events, so invalidate events queries
            dispatch(scoutApi.util.invalidateTags(["Event"]));
          }
          break;
        case "UPDATE":
          operation = EnumRealtimeOperation.UPDATE;
          if (data.record && invalidateRTKQuery) {
            console.log(
              "[Tags] Tag updated, invalidating RTK Query cache:",
              data.record,
            );
            // Invalidate events queries since tags are embedded in events
            dispatch(scoutApi.util.invalidateTags(["Event"]));
          }
          break;
        case "DELETE":
          operation = EnumRealtimeOperation.DELETE;
          if (data.old_record && invalidateRTKQuery) {
            console.log(
              "[Tags] Tag deleted, invalidating RTK Query cache:",
              data.old_record,
            );
            // Invalidate events queries since tags are embedded in events
            dispatch(scoutApi.util.invalidateTags(["Event"]));
          }
          break;
        default:
          return;
      }

      const realtimeData: RealtimeData<ITagPrettyLocation> = {
        data: tagData,
        operation,
      };

      console.log(
        `[scout-core realtime] TAG ${data.operation} received:`,
        JSON.stringify(realtimeData),
      );

      setLatestTagUpdate(realtimeData);
    },
    [dispatch, invalidateRTKQuery],
  );

  // Clear latest update
  const clearLatestUpdate = useCallback(() => {
    setLatestTagUpdate(null);
  }, []);

  const cleanupChannels = () => {
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];
  };

  const createTagsChannel = (herdId: string): RealtimeChannel => {
    return scoutSupabase
      .channel(`${herdId}-tags`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleTagBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Tags] ✅ Connected to herd ${herdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.warn(`[Tags] 🟡 Failed to connect to herd ${herdId}`);
        }
      });
  };

  useEffect(() => {
    cleanupChannels();

    // Clear previous update when switching herds
    clearLatestUpdate();

    // Create tags channel for active herd
    if (activeHerdId) {
      const channel = createTagsChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearLatestUpdate]);

  return [latestTagUpdate, clearLatestUpdate];
}
