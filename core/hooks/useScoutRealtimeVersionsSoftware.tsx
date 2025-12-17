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
): [RealtimeData<IVersionsSoftware> | null, () => void] {
  const channels = useRef<RealtimeChannel[]>([]);
  const [latestVersionUpdate, setLatestVersionUpdate] =
    useState<RealtimeData<IVersionsSoftware> | null>(null);

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
        `[scout-core realtime] VERSIONS_SOFTWARE ${data.operation} received for version ${versionData.system}@${versionData.version}:`,
        JSON.stringify(versionData),
      );

      const realtimeData: RealtimeData<IVersionsSoftware> = {
        data: versionData,
        operation,
      };

      setLatestVersionUpdate(realtimeData);
    },
    [],
  );

  // Clear latest update
  const clearLatestUpdate = useCallback(() => {
    setLatestVersionUpdate(null);
  }, []);

  useEffect(() => {
    if (!scoutSupabase) return;

    // Clean up existing channels
    channels.current.forEach((channel) => scoutSupabase.removeChannel(channel));
    channels.current = [];

    // Clear previous update when setting up
    clearLatestUpdate();

    // Create versions_software channel
    const channel = scoutSupabase
      .channel("versions_software_changes", { config: { private: true } })
      .on("broadcast", { event: "*" }, handleVersionsSoftwareBroadcast)
      .subscribe((status) => {
        if (status === "SUBSCRIBED") {
          console.log(
            "[VERSIONS_SOFTWARE] ✅ Connected to software versions broadcasts",
          );
        } else if (status === "CHANNEL_ERROR") {
          console.warn(
            "[VERSIONS_SOFTWARE] 🟡 Failed to connect to software versions broadcasts",
          );
        }
      });

    channels.current.push(channel);

    return () => {
      channels.current.forEach((ch) => scoutSupabase.removeChannel(ch));
      channels.current = [];
    };
  }, [scoutSupabase, handleVersionsSoftwareBroadcast, clearLatestUpdate]);

  return [latestVersionUpdate, clearLatestUpdate];
}
