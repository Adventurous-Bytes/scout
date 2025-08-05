import { SupabaseClient } from "@supabase/supabase-js";

import { LABELS } from "../constants/annotator";
import { get_devices_by_herd } from "../helpers/devices";
import { server_get_total_events_by_herd } from "../helpers/events";
import { EnumSessionsVisibility } from "./events";
import { server_get_plans_by_herd } from "../helpers/plans";
import { server_get_layers_by_herd } from "../helpers/layers";
import { server_get_events_and_tags_for_devices_batch } from "../helpers/tags";
import { server_get_users_with_herd_access } from "../helpers/users";
import {
  IDevice,
  IEventWithTags,
  IHerd,
  IPlan,
  ILayer,
  IUserAndRole,
  IZoneWithActions,
  ISessionWithCoordinates,
} from "../types/db";
import { EnumWebResponse } from "./requests";
import { server_get_more_zones_and_actions_for_herd } from "../helpers/zones";
import { server_list_api_keys_batch } from "../api_keys/actions";
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
  layers: ILayer[] = [];
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
    sessions: ISessionWithCoordinates[] = [],
    layers: ILayer[] = []
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
    this.layers = layers;
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
      layers: this.layers,
    };
  }
  static async from_herd(
    herd: IHerd,
    client: SupabaseClient
  ): Promise<HerdModule> {
    const startTime = Date.now();

    try {
      // load devices
      let response_new_devices = await get_devices_by_herd(herd.id, client);
      if (
        response_new_devices.status == EnumWebResponse.ERROR ||
        !response_new_devices.data
      ) {
        console.warn(`[HerdModule] No devices found for herd ${herd.id}`);
        return new HerdModule(herd, [], [], Date.now());
      }
      const new_devices = response_new_devices.data;

      // get api keys and events for all devices in batch
      let recent_events_batch: { [device_id: number]: IEventWithTags[] } = {};
      if (new_devices.length > 0) {
        try {
          const device_ids = new_devices.map((device) => device.id ?? 0);

          // Load API keys and events in parallel
          const [api_keys_batch, events_response] = await Promise.all([
            server_list_api_keys_batch(device_ids),
            server_get_events_and_tags_for_devices_batch(device_ids, 1),
          ]);

          // Assign API keys to devices
          for (let i = 0; i < new_devices.length; i++) {
            const device_id = new_devices[i].id ?? 0;
            new_devices[i].api_keys_scout = api_keys_batch[device_id] || [];
          }

          // Process events response
          if (
            events_response.status === EnumWebResponse.SUCCESS &&
            events_response.data
          ) {
            recent_events_batch = events_response.data;
          }
        } catch (error) {
          console.error(`[HerdModule] Batch load error:`, error);
          // Continue without API keys and events
        }
      }

      // Run all remaining requests in parallel with individual error handling
      const [
        res_zones,
        res_user_roles,
        total_event_count,
        res_plans,
        res_sessions,
        res_layers,
      ] = await Promise.allSettled([
        server_get_more_zones_and_actions_for_herd(herd.id, 0, 10).catch(
          (error) => {
            console.warn(
              `[HerdModule] Failed to get zones and actions:`,
              error
            );
            return { status: EnumWebResponse.ERROR, data: null };
          }
        ),
        server_get_users_with_herd_access(herd.id).catch((error) => {
          console.warn(`[HerdModule] Failed to get user roles:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_total_events_by_herd(
          herd.id,
          EnumSessionsVisibility.Exclude
        ).catch((error) => {
          console.warn(`[HerdModule] Failed to get total events count:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_plans_by_herd(herd.id).catch((error) => {
          console.warn(`[HerdModule] Failed to get plans:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        getSessionsByHerdId(client, herd.id).catch((error) => {
          console.warn(`[HerdModule] Failed to get sessions:`, error);
          return [];
        }),
        server_get_layers_by_herd(herd.id).catch((error) => {
          console.warn(`[HerdModule] Failed to get layers:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
      ]);

      // Assign recent events to devices from batch results
      for (let i = 0; i < new_devices.length; i++) {
        try {
          const device_id = new_devices[i].id ?? 0;
          const events = recent_events_batch[device_id];
          if (events) {
            new_devices[i].recent_events = events;
          }
        } catch (error) {
          console.warn(
            `Failed to process events for device ${new_devices[i].id}:`,
            error
          );
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
      const layers =
        res_layers.status === "fulfilled" && res_layers.value?.data
          ? res_layers.value.data
          : [];

      // TODO: store in DB and retrieve on load?
      const newLabels = LABELS;

      const endTime = Date.now();
      const loadTime = endTime - startTime;
      console.log(
        `[HerdModule] Loaded herd ${herd.slug} in ${loadTime}ms (${new_devices.length} devices)`
      );

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
        sessions,
        layers
      );
    } catch (error) {
      const endTime = Date.now();
      const loadTime = endTime - startTime;
      console.error(
        `[HerdModule] Critical error in HerdModule.from_herd (${loadTime}ms):`,
        error
      );
      // Return a minimal but valid HerdModule instance to prevent complete failure
      return new HerdModule(
        herd,
        [],
        [],
        Date.now(),
        null,
        0,
        0,
        0,
        [],
        [],
        [],
        [],
        []
      );
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
  layers: ILayer[];
}
