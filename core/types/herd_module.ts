import { SupabaseClient } from "@supabase/supabase-js";

import { LABELS } from "../constants/annotator";
import { get_devices_by_herd } from "../helpers/devices";
import { server_get_total_events_by_herd } from "../helpers/events";
import { EnumSessionsVisibility } from "./events";
import { server_get_plans_by_herd } from "../helpers/plans";
import { server_get_layers_by_herd } from "../helpers/layers";
import { server_get_providers_by_herd } from "../helpers/providers";
import { server_get_events_and_tags_for_devices_batch } from "../helpers/tags";
import { server_get_users_with_herd_access } from "../helpers/users";
import {
  IDevice,
  IEventWithTags,
  IEventAndTagsPrettyLocation,
  IHerd,
  IPlan,
  ILayer,
  IProvider,
  IUserAndRole,
  IZoneWithActions,
  ISessionWithCoordinates,
  IArtifactWithMediaUrl,
} from "../types/db";

import { EnumWebResponse } from "./requests";
import { server_get_more_zones_and_actions_for_herd } from "../helpers/zones";
import { server_list_api_keys_batch } from "../api_keys/actions";
import { server_get_sessions_by_herd_id } from "../helpers/sessions";
import {
  server_get_artifacts_by_herd,
  server_get_total_artifacts_by_herd,
} from "../helpers/artifacts";
export enum EnumHerdModulesLoadingState {
  NOT_LOADING = "NOT_LOADING",
  LOADING = "LOADING",
  SUCCESSFULLY_LOADED = "SUCCESSFULLY_LOADED",
  UNSUCCESSFULLY_LOADED = "UNSUCCESSFULLY_LOADED",
}

export class HerdModule {
  herd: IHerd;
  devices: IDevice[];
  events: IEventWithTags[];
  zones: IZoneWithActions[];
  sessions: ISessionWithCoordinates[];
  artifacts: IArtifactWithMediaUrl[];
  timestamp_last_refreshed: number;
  user_roles: IUserAndRole[] | null = null;
  events_page_index: number = 0;
  total_events: number = 0;
  total_events_with_filters: number = 0;
  total_artifacts: number = 0;
  labels: string[] = [];
  plans: IPlan[] = [];
  layers: ILayer[] = [];
  providers: IProvider[] = [];
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
    layers: ILayer[] = [],
    providers: IProvider[] = [],
    artifacts: IArtifactWithMediaUrl[] = [],
    total_artifacts: number = 0,
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
    this.providers = providers;
    this.artifacts = artifacts;
    this.total_artifacts = total_artifacts;
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
      providers: this.providers,
      artifacts: this.artifacts,
      total_artifacts: this.total_artifacts,
    };
  }
  static async from_herd(
    herd: IHerd,
    client: SupabaseClient,
  ): Promise<HerdModule> {
    const startTime = Date.now();

    try {
      // Start loading herd-level data in parallel with devices
      const herdLevelPromises = Promise.allSettled([
        server_get_more_zones_and_actions_for_herd(herd.id, 0, 10).catch(
          (error) => {
            console.warn(
              `[HerdModule] Failed to get zones and actions:`,
              error,
            );
            return { status: EnumWebResponse.ERROR, data: null };
          },
        ),
        server_get_users_with_herd_access(herd.id, client).catch((error) => {
          console.warn(`[HerdModule] Failed to get user roles:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_total_events_by_herd(
          herd.id,
          EnumSessionsVisibility.Exclude,
        ).catch((error) => {
          console.warn(`[HerdModule] Failed to get total events count:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_plans_by_herd(herd.id).catch((error) => {
          console.warn(`[HerdModule] Failed to get plans:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_sessions_by_herd_id(herd.id).catch((error) => {
          console.warn(`[HerdModule] Failed to get sessions:`, error);
          return {
            status: EnumWebResponse.ERROR,
            data: [],
            msg: error.message,
          };
        }),
        server_get_layers_by_herd(herd.id).catch((error) => {
          console.warn(`[HerdModule] Failed to get layers:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_providers_by_herd(herd.id).catch((error) => {
          console.warn(`[HerdModule] Failed to get providers:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_artifacts_by_herd(herd.id, 50, 0).catch((error) => {
          console.warn(`[HerdModule] Failed to get artifacts:`, error);
          return { status: EnumWebResponse.ERROR, data: null };
        }),
        server_get_total_artifacts_by_herd(herd.id).catch((error) => {
          console.warn(
            `[HerdModule] Failed to get total artifacts count:`,
            error,
          );
          return { status: EnumWebResponse.ERROR, data: null };
        }),
      ]);

      // Load devices
      const devicesPromise = get_devices_by_herd(herd.id, client);

      // Wait for both devices and herd-level data
      const [deviceResponse, herdLevelResults] = await Promise.all([
        devicesPromise,
        herdLevelPromises,
      ]);

      // Check devices response
      if (
        deviceResponse.status == EnumWebResponse.ERROR ||
        !deviceResponse.data
      ) {
        console.warn(`[HerdModule] No devices found for herd ${herd.id}`);
        return new HerdModule(herd, [], [], Date.now());
      }
      const new_devices = deviceResponse.data;

      // Load API keys for devices if we have any
      if (new_devices.length > 0) {
        try {
          const device_ids = new_devices.map((device) => device.id ?? 0);
          const api_keys_batch = await server_list_api_keys_batch(device_ids);

          // Assign API keys to devices
          for (let i = 0; i < new_devices.length; i++) {
            const device_id = new_devices[i].id ?? 0;
            new_devices[i].api_keys_scout = api_keys_batch[device_id] || [];
          }
        } catch (error) {
          console.error(`[HerdModule] Failed to load API keys:`, error);
          // Continue without API keys
        }
      }

      // Extract herd-level data with safe fallbacks
      const [
        res_zones,
        res_user_roles,
        total_event_count,
        res_plans,
        res_sessions,
        res_layers,
        res_providers,
        res_artifacts,
        total_artifact_count,
      ] = herdLevelResults;

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
        res_sessions.status === "fulfilled" && res_sessions.value?.data
          ? res_sessions.value.data
          : [];
      const layers =
        res_layers.status === "fulfilled" && res_layers.value?.data
          ? res_layers.value.data
          : [];
      const providers =
        res_providers.status === "fulfilled" && res_providers.value?.data
          ? res_providers.value.data
          : [];
      const artifacts =
        res_artifacts.status === "fulfilled" && res_artifacts.value?.data
          ? res_artifacts.value.data
          : [];
      const total_artifacts =
        total_artifact_count.status === "fulfilled" &&
        total_artifact_count.value?.data
          ? total_artifact_count.value.data
          : 0;

      // TODO: store in DB and retrieve on load?
      const newLabels = LABELS;

      const endTime = Date.now();
      const loadTime = endTime - startTime;
      console.log(
        `[HerdModule] Loaded herd ${herd.slug} in ${loadTime}ms (${new_devices.length} devices)`,
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
        layers,
        providers,
        artifacts,
        total_artifacts,
      );
    } catch (error) {
      const endTime = Date.now();
      const loadTime = endTime - startTime;
      console.error(
        `[HerdModule] Critical error in HerdModule.from_herd (${loadTime}ms):`,
        error,
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
        [],
        [],
        [],
        0,
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
  providers: IProvider[];
  artifacts: IArtifactWithMediaUrl[];
  total_artifacts: number;
}

export interface IHerdModulesResponse {
  data: IHerdModule[];
  time_finished: number;
  server_processing_time_ms: number;
}

export interface IHerdModulesResponseWithStatus {
  status: EnumWebResponse;
  msg: string;
  data: IHerdModule[] | null;
  time_finished: number; // When server finished processing
  time_sent: number; // When server actually sent the response
  server_processing_time_ms: number;
}
