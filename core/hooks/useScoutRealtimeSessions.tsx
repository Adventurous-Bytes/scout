"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import {
  addSessionToStore,
  deleteSessionFromStore,
  updateSessionInStore,
} from "../store/scout";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { ISessionWithCoordinates } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: ISessionWithCoordinates;
    old_record?: ISessionWithCoordinates;
  };
};

export function useScoutRealtimeSessions(
  scoutSupabase: SupabaseClient<Database>,
  shouldUpdateGlobalStateOnChanges: boolean,
): RealtimeData<ISessionWithCoordinates>[] {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();
  const [newSessionItems, setNewSessionItems] = useState<
    RealtimeData<ISessionWithCoordinates>[]
  >([]);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  // Session broadcast handler
  const handleSessionBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      console.log("[Sessions] Broadcast received:", payload.payload.operation);

      const data = payload.payload;

      const sessionData = data.record || data.old_record;
      if (!sessionData) return;

      let operation: EnumRealtimeOperation;
      switch (data.operation) {
        case "INSERT":
          operation = EnumRealtimeOperation.INSERT;
          if (data.record && shouldUpdateGlobalStateOnChanges) {
            console.log("[Sessions] New session received:", data.record);
            dispatch(addSessionToStore(data.record));
          }
          break;
        case "UPDATE":
          operation = EnumRealtimeOperation.UPDATE;
          if (data.record && shouldUpdateGlobalStateOnChanges) {
            dispatch(updateSessionInStore(data.record));
          }
          break;
        case "DELETE":
          operation = EnumRealtimeOperation.DELETE;
          if (data.old_record && shouldUpdateGlobalStateOnChanges) {
            dispatch(deleteSessionFromStore(data.old_record));
          }
          break;
        default:
          return;
      }

      const realtimeData: RealtimeData<ISessionWithCoordinates> = {
        data: sessionData,
        operation,
      };

      console.log(
        `[scout-core realtime] SESSION ${data.operation} received:`,
        JSON.stringify(realtimeData),
      );

      setNewSessionItems((prev) => [realtimeData, ...prev]);
    },
    [dispatch],
  );

  // Clear new items when herd changes
  const clearNewItems = useCallback(() => {
    setNewSessionItems([]);
  }, []);

  const cleanupChannels = () => {
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];
  };

  const createSessionsChannel = (herdId: string): RealtimeChannel => {
    return scoutSupabase
      .channel(`${herdId}-sessions`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handleSessionBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Sessions] ✅ Connected to herd ${herdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.warn(`[Sessions] 🟡 Failed to connect to herd ${herdId}`);
        }
      });
  };

  useEffect(() => {
    cleanupChannels();

    // Clear previous items when switching herds
    clearNewItems();

    // Create sessions channel for active herd
    if (activeHerdId) {
      const channel = createSessionsChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearNewItems]);

  return newSessionItems;
}
