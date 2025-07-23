import { SupabaseClient } from "@supabase/supabase-js";

import { LABELS } from "../constants/annotator";
import { get_devices_by_herd } from "../helpers/devices";
import {
  EnumSessionsVisibility,
  server_get_total_events_by_herd,
} from "../helpers/events";
import { server_get_plans_by_herd } from "../helpers/plans";
import { server_get_events_and_tags_for_device } from "../helpers/tags";
import { server_get_users_with_herd_access } from "../helpers/users";
import {
  IDevice,
  IEventWithTags,
  IHerd,
  IPlan,
  IUserAndRole,
  IZoneWithActions,
  ISessionWithCoordinates,
} from "../types/db";
import { EnumWebResponse } from "./requests";
import { server_get_more_zones_and_actions_for_herd } from "../helpers/zones";
import { server_list_api_keys } from "../api_keys/actions";
import { getSessionsByHerdId } from "../helpers/sessions";

export class HerdModule {
  herd: IHerd;
  devices: IDevice[];
  events: IEventWithTags[];
  zones: IZoneWithActions[];
  sessions: ISessionWithCoordinates[];
  timestamp_last_refreshed: number;
  user_roles: IUserAndRole[] | null = null;
  events_page_index: number = 0;
  total_events: number = 0;
  total_events_with_filters: number = 0;
  labels: string[] = [];
  plans: IPlan[] = [];
  constructor(
    herd: IHerd,
    devices: IDevice[],
    events: IEventWithTags[],
    timestamp_last_refreshed: number,
    user_roles: IUserAndRole[] | null = null,
    events_page_index: number = 0,
    total_events: number = 0,
    total_events_with_filters: number = 0,
    labels: string[] = [],
    plans: IPlan[] = [],
    zones: IZoneWithActions[] = [],
    sessions: ISessionWithCoordinates[] = []
  ) {
    this.herd = herd;
    this.devices = devices;
    this.events = events;
    this.timestamp_last_refreshed = timestamp_last_refreshed;
    this.user_roles = user_roles;
    this.events_page_index = events_page_index;
    this.total_events = total_events;
    this.total_events_with_filters = total_events_with_filters;
    this.labels = labels;
    this.plans = plans;
    this.zones = zones;
    this.sessions = sessions;
  }
  to_serializable(): IHerdModule {
    return {
      herd: this.herd,
      devices: this.devices,
      events: this.events,
      timestamp_last_refreshed: this.timestamp_last_refreshed,
      user_roles: this.user_roles,
      events_page_index: this.events_page_index,
      total_events: this.total_events,
      total_events_with_filters: this.total_events_with_filters,
      labels: this.labels,
      plans: this.plans,
      zones: this.zones,
      sessions: this.sessions,
    };
  }
  static async from_herd(
    herd: IHerd,
    client: SupabaseClient
  ): Promise<HerdModule> {
    try {
      // load devices
      let response_new_devices = await get_devices_by_herd(herd.id, client);
      if (
        response_new_devices.status == EnumWebResponse.ERROR ||
        !response_new_devices.data
      ) {
        console.warn("No devices found for herd");
        return new HerdModule(herd, [], [], Date.now());
      }
      const new_devices = response_new_devices.data;

      // get api keys for each device... run requests in parallel
      if (new_devices.length > 0) {
        try {
          let api_keys_promises = new_devices.map((device) =>
            server_list_api_keys(device.id?.toString() ?? "").catch((error) => {
              console.warn(
                `Failed to get API keys for device ${device.id}:`,
                error
              );
              return undefined;
            })
          );
          let api_keys = await Promise.all(api_keys_promises);
          for (let i = 0; i < new_devices.length; i++) {
            new_devices[i].api_keys_scout = api_keys[i];
          }
        } catch (error) {
          console.warn("Failed to load API keys for devices:", error);
          // Continue without API keys
        }
      }

      // get recent events for each device... run requests in parallel
      let recent_events_promises = new_devices.map((device) =>
        server_get_events_and_tags_for_device(device.id ?? 0).catch((error) => {
          console.warn(`Failed to get events for device ${device.id}:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        })
      );

      // Run all requests in parallel with individual error handling
      const [
        recent_events,
        res_zones,
        res_user_roles,
        total_event_count,
        res_plans,
        res_sessions,
      ] = await Promise.allSettled([
        Promise.all(recent_events_promises),
        server_get_more_zones_and_actions_for_herd(herd.id, 0, 10).catch(
          (error) => {
            console.warn("Failed to get zones and actions:", error);
            return { status: EnumWebResponse.ERROR, data: null };
          }
        ),
        server_get_users_with_herd_access(herd.id).catch((error) => {
          console.warn("Failed to get user roles:", error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_total_events_by_herd(
          herd.id,
          EnumSessionsVisibility.Exclude
        ).catch((error) => {
          console.warn("Failed to get total events count:", error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_plans_by_herd(herd.id).catch((error) => {
          console.warn("Failed to get plans:", error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        getSessionsByHerdId(client, herd.id).catch((error) => {
          console.warn("Failed to get sessions:", error);
          return [];
        }),
      ]);

      // Process recent events with error handling
      if (recent_events.status === "fulfilled") {
        for (let i = 0; i < new_devices.length; i++) {
          try {
            let x: IEventWithTags[] | null = recent_events.value[i]?.data;
            if (
              recent_events.value[i]?.status == EnumWebResponse.SUCCESS &&
              x
            ) {
              new_devices[i].recent_events = x;
            }
          } catch (error) {
            console.warn(
              `Failed to process events for device ${new_devices[i].id}:`,
              error
            );
          }
        }
      }

      // Extract data with safe fallbacks
      const zones =
        res_zones.status === "fulfilled" && res_zones.value?.data
          ? res_zones.value.data
          : [];
      const user_roles =
        res_user_roles.status === "fulfilled" && res_user_roles.value?.data
          ? res_user_roles.value.data
          : null;
      const total_events =
        total_event_count.status === "fulfilled" &&
        total_event_count.value?.data
          ? total_event_count.value.data
          : 0;
      const plans =
        res_plans.status === "fulfilled" && res_plans.value?.data
          ? res_plans.value.data
          : [];
      const sessions =
        res_sessions.status === "fulfilled" ? res_sessions.value : [];

      // TODO: store in DB and retrieve on load?
      const newLabels = LABELS;
      return new HerdModule(
        herd,
        new_devices,
        [],
        Date.now(),
        user_roles,
        0,
        total_events,
        total_events,
        newLabels,
        plans,
        zones,
        sessions
      );
    } catch (error) {
      console.error("Critical error in HerdModule.from_herd:", error);
      // Return a minimal but valid HerdModule instance to prevent complete failure
      return new HerdModule(herd, [], [], Date.now());
    }
  }
}

export interface IHerdModule {
  herd: IHerd;
  devices: IDevice[];
  events: IEventWithTags[];
  timestamp_last_refreshed: number;
  user_roles: IUserAndRole[] | null;
  events_page_index: number;
  total_events: number;
  total_events_with_filters: number;
  labels: string[];
  plans: IPlan[];
  zones: IZoneWithActions[];
  sessions: ISessionWithCoordinates[];
}
