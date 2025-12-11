"use client";

import { useAppDispatch } from "../store/hooks";
import { useSelector } from "react-redux";
import { useEffect, useRef, useCallback, useState } from "react";
import { addPlan, deletePlan, updatePlan } from "../store/scout";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IPlan } from "../types/db";
import { RootState } from "../store/scout";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: IPlan;
    old_record?: IPlan;
  };
};

export function useScoutRealtimePlans(
  scoutSupabase: SupabaseClient<Database>,
  shouldUpdateGlobalStateOnChanges: boolean,
): RealtimeData<IPlan>[] {
  const channels = useRef<RealtimeChannel[]>([]);
  const dispatch = useAppDispatch();
  const [newPlanItems, setNewPlanItems] = useState<RealtimeData<IPlan>[]>([]);

  const activeHerdId = useSelector(
    (state: RootState) => state.scout.active_herd_id,
  );

  // Plan broadcast handler
  const handlePlanBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      console.log("[Plans] Broadcast received:", payload.payload.operation);

      const data = payload.payload;

      const planData = data.record || data.old_record;
      if (!planData) return;

      let operation: EnumRealtimeOperation;
      switch (data.operation) {
        case "INSERT":
          operation = EnumRealtimeOperation.INSERT;
          if (data.record && shouldUpdateGlobalStateOnChanges) {
            console.log("[Plans] New plan received:", data.record);
            dispatch(addPlan(data.record));
          }
          break;
        case "UPDATE":
          operation = EnumRealtimeOperation.UPDATE;
          if (data.record && shouldUpdateGlobalStateOnChanges) {
            dispatch(updatePlan(data.record));
          }
          break;
        case "DELETE":
          operation = EnumRealtimeOperation.DELETE;
          if (data.old_record && shouldUpdateGlobalStateOnChanges) {
            dispatch(deletePlan(data.old_record));
          }
          break;
        default:
          return;
      }

      const realtimeData: RealtimeData<IPlan> = {
        data: planData,
        operation,
      };

      console.log(
        `[scout-core realtime] PLAN ${data.operation} received:`,
        JSON.stringify(realtimeData),
      );

      setNewPlanItems((prev) => [realtimeData, ...prev]);
    },
    [dispatch],
  );

  // Clear new items when herd changes
  const clearNewItems = useCallback(() => {
    setNewPlanItems([]);
  }, []);

  const cleanupChannels = () => {
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];
  };

  const createPlansChannel = (herdId: string): RealtimeChannel => {
    return scoutSupabase
      .channel(`${herdId}-plans`, { config: { private: true } })
      .on("broadcast", { event: "*" }, handlePlanBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(`[Plans] ✅ Connected to herd ${herdId}`);
        } else if (status === "CHANNEL_ERROR") {
          console.warn(`[Plans] 🟡 Failed to connect to herd ${herdId}`);
        }
      });
  };

  useEffect(() => {
    cleanupChannels();

    // Clear previous items when switching herds
    clearNewItems();

    // Create plans channel for active herd
    if (activeHerdId) {
      const channel = createPlansChannel(activeHerdId);
      channels.current.push(channel);
    }

    return cleanupChannels;
  }, [activeHerdId, clearNewItems]);

  return newPlanItems;
}
