import { SupabaseClient } from "@supabase/supabase-js";

import { LABELS } from "../constants/annotator";
import { get_devices_by_herd } from "../helpers/devices";
import { server_get_total_events_by_herd } from "../helpers/events";
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
} from "../types/db";
import { EnumWebResponse } from "./requests";
import { server_get_more_zones_and_actions_for_herd } from "../helpers/zones";
import { server_list_api_keys } from "../api_keys/actions";

export class HerdModule {
  herd: IHerd;
  devices: IDevice[];
  events: IEventWithTags[];
  zones: IZoneWithActions[];
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
    zones: IZoneWithActions[] = []
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
    };
  }
  static async from_herd(
    herd: IHerd,
    client: SupabaseClient
  ): Promise<HerdModule> {
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
      let api_keys_promises = new_devices.map((device) =>
        server_list_api_keys(device.id?.toString() ?? "")
      );
      let api_keys = await Promise.all(api_keys_promises);
      for (let i = 0; i < new_devices.length; i++) {
        new_devices[i].api_keys_scout = api_keys[i];
      }
    }
    // get recent events for each device... run requests in parallel

    let recent_events_promises = new_devices.map((device) =>
      server_get_events_and_tags_for_device(device.id ?? 0)
    );

    // Run all requests in parallel
    const [
      recent_events,
      res_zones,
      res_user_roles,
      total_event_count,
      res_plans,
    ] = await Promise.all([
      Promise.all(recent_events_promises),
      server_get_more_zones_and_actions_for_herd(herd.id, 0, 10),
      server_get_users_with_herd_access(herd.id),
      server_get_total_events_by_herd(herd.id),
      server_get_plans_by_herd(herd.id),
    ]);
    for (let i = 0; i < new_devices.length; i++) {
      let x: IEventWithTags[] | null = recent_events[i].data;
      if (recent_events[i].status == EnumWebResponse.SUCCESS && x) {
        new_devices[i].recent_events = x;
      }
    }

    // TODO: store in DB and retrieve on load?
    const newLabels = LABELS;
    return new HerdModule(
      herd,
      new_devices,
      [],
      Date.now(),
      res_user_roles.data,
      0,
      total_event_count.data ? total_event_count.data : 0,
      total_event_count.data ? total_event_count.data : 0,
      newLabels,
      res_plans.data ? res_plans.data : [],
      res_zones.data ? res_zones.data : []
    );
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
}
