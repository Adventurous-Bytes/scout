"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import { updateEventValuesForHerdModule } from "../store/scout";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IEventAndTagsPrettyLocation } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

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
  shouldUpdateGlobalStateOnChanges: boolean,
): RealtimeData<IEventAndTagsPrettyLocation>[] {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();
  const [newEventItems, setNewEventItems] = useState<
    RealtimeData<IEventAndTagsPrettyLocation>[]
  >([]);

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
      // TODO: UNCOMMENT GLOBAL STORE OPERATIONS IF OKAY WITH FREQUENT
      switch (data.operation) {
        case "INSERT":
          operation = EnumRealtimeOperation.INSERT;
          if (data.record && activeHerdId && shouldUpdateGlobalStateOnChanges) {
            console.log("[Events] New event received:", data.record);
            // For events, we need to update the event values in the herd module
            dispatch(
              updateEventValuesForHerdModule({
                herd_id: activeHerdId,
                events: [data.record],
              }),
            );
          }
          break;
        case "UPDATE":
          operation = EnumRealtimeOperation.UPDATE;
          if (data.record && activeHerdId && shouldUpdateGlobalStateOnChanges) {
            console.log("[Events] Event updated:", data.record);
            dispatch(
              updateEventValuesForHerdModule({
                herd_id: activeHerdId,
                events: [data.record],
              }),
            );
          }
          break;
        case "DELETE":
          operation = EnumRealtimeOperation.DELETE;
          if (
            data.old_record &&
            activeHerdId &&
            shouldUpdateGlobalStateOnChanges
          ) {
            console.log("[Events] Event deleted:", data.old_record);
            // TODO: WRITE DELETION STORE ACTION
            console.log(
              "[Events] Event deletion detected - manual refresh may be needed",
            );
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

      setNewEventItems((prev) => [realtimeData, ...prev]);
    },
    [dispatch, activeHerdId],
  );

  // Clear new items when herd changes
  const clearNewItems = useCallback(() => {
    setNewEventItems([]);
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

    // Clear previous items when switching herds
    clearNewItems();

    // Create events channel for active herd
    if (activeHerdId) {
      const channel = createEventsChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearNewItems]);

  return newEventItems;
}
