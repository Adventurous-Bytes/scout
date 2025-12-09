"use client";

import { useEffect, useRef, useCallback, useState } from "react";
import { SupabaseClient, RealtimeChannel } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { IVersionsSoftware } from "../types/db";
import { RealtimeData, EnumRealtimeOperation } from "../types/realtime";

type BroadcastPayload = {
  type: "broadcast";
  event: string;
  payload: {
    operation: "INSERT" | "UPDATE" | "DELETE";
    table: string;
    schema: string;
    record?: IVersionsSoftware;
    old_record?: IVersionsSoftware;
  };
};

export function useScoutRealtimeVersionsSoftware(
  scoutSupabase: SupabaseClient<Database>,
): RealtimeData<IVersionsSoftware>[] {
  const channels = useRef<RealtimeChannel[]>([]);
  const [newVersionsItems, setNewVersionsItems] = useState<
    RealtimeData<IVersionsSoftware>[]
  >([]);

  // Handle versions software broadcasts
  const handleVersionsSoftwareBroadcast = useCallback(
    (payload: BroadcastPayload) => {
      const { event, payload: data } = payload;
      const versionData = data.record || data.old_record;

      if (!versionData) {
        return;
      }

      let operation: EnumRealtimeOperation;
      switch (data.operation) {
        case "INSERT":
          operation = EnumRealtimeOperation.INSERT;
          break;
        case "UPDATE":
          operation = EnumRealtimeOperation.UPDATE;
          break;
        case "DELETE":
          operation = EnumRealtimeOperation.DELETE;
          break;
        default:
          return;
      }

      console.log(
        `[VERSIONS_SOFTWARE] ${data.operation} received for version ${versionData.system}@${versionData.version}:`,
        JSON.stringify(versionData),
      );

      const realtimeData: RealtimeData<IVersionsSoftware> = {
        data: versionData,
        operation,
      };

      setNewVersionsItems((prev) => [realtimeData, ...prev]);
    },
    [],
  );

  // Clear new items
  const clearNewItems = useCallback(() => {
    setNewVersionsItems([]);
  }, []);

  useEffect(() => {
    if (!scoutSupabase) return;

    // Clean up existing channels
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];

    // Clear previous items when setting up
    clearNewItems();

    // Create versions_software channel
    const channel = scoutSupabase
      .channel("versions_software_changes", { config: { private: true } })
      .on("broadcast", { event: "*" }, handleVersionsSoftwareBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log("[VERSIONS_SOFTWARE] ✅ Connected to software versions broadcasts");
        } else if (status === "CHANNEL_ERROR") {
          console.warn("[VERSIONS_SOFTWARE] 🟡 Failed to connect to software versions broadcasts");
        }
      });

    channels.current.push(channel);

    return () => {
      channels.current.forEach((ch) => scoutSupabase.removeChannel(ch));
      channels.current = [];
    };
  }, [scoutSupabase, handleVersionsSoftwareBroadcast, clearNewItems]);

  return newVersionsItems;
}
