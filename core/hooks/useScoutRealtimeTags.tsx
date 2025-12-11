"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import { addTag, deleteTag, updateTag } from "../store/scout";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { ITagPrettyLocation } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

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
  shouldUpdateGlobalStateOnChanges: boolean,
): RealtimeData<ITagPrettyLocation>[] {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();
  const [newTagItems, setNewTagItems] = useState<
    RealtimeData<ITagPrettyLocation>[]
  >([]);

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
          if (data.record && shouldUpdateGlobalStateOnChanges) {
            console.log("[Tags] New tag received:", data.record);
            dispatch(addTag(data.record));
          }
          break;
        case "UPDATE":
          operation = EnumRealtimeOperation.UPDATE;
          if (data.record && shouldUpdateGlobalStateOnChanges) {
            console.log("[Tags] Tag updated:", data.record);
            dispatch(updateTag(data.record));
          }
          break;
        case "DELETE":
          operation = EnumRealtimeOperation.DELETE;
          if (data.old_record && shouldUpdateGlobalStateOnChanges) {
            console.log("[Tags] Tag deleted:", data.old_record);
            dispatch(deleteTag(data.old_record));
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

      setNewTagItems((prev) => [realtimeData, ...prev]);
    },
    [dispatch],
  );

  // Clear new items when herd changes
  const clearNewItems = useCallback(() => {
    setNewTagItems([]);
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

    // Clear previous items when switching herds
    clearNewItems();

    // Create tags channel for active herd
    if (activeHerdId) {
      const channel = createTagsChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearNewItems]);

  return newTagItems;
}
