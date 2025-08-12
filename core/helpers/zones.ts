import { newServerClient } from "../supabase/server";
import { IAction, IZoneWithActions } from "../types/db";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";
import { IZonesAndActionsPrettyLocation } from "../types/db";

/**
 * Get more zones and actions for a herd
 * @param herd_id - The ID of the herd to get zones and actions for
 * @param offset - The offset to start the query from
 * @param page_count - The number of zones and actions to return
 * @returns A list of zones and actions
 * @throws An error if the zones or actions are not fetched
 */
export async function server_get_more_zones_and_actions_for_herd(
  herd_id: number,
  offset: number,
  page_count: number = 10
): Promise<IWebResponseCompatible<IZoneWithActions[]>> {
  const from = offset * page_count;
  const to = from + page_count - 1;
  const supabase = await newServerClient();
  // make rpc call to get_events_with_tags_for_herd(herd_id, offset, limit)
  const { data, error } = await supabase.rpc("get_zones_and_actions_for_herd", {
    herd_id_caller: herd_id,
    offset_caller: from,
    limit_caller: page_count,
  });
  if (error) {
    console.warn("Error fetching zones and actions for herd:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }
  return IWebResponse.success(
    data.map((zone: any) => ({
      ...zone,
      actions: zone.actions ?? [],
    }))
  ).to_compatible();
}

/**
 * Create zones and actions for a herd
 * @param zones - The zones to create
 * @returns A list of zones and actions
 * @throws An error if the zones or actions are not created
 */
export async function server_create_zones_with_actions(
  zones: IZoneWithActions[]
): Promise<IWebResponseCompatible<IZoneWithActions[]>> {
  // loop through plans and format
  let actions: IAction[] = zones.flatMap((zone) => zone.actions);
  let formatted_zones = zones.map((zone) => {
    let formatted_zone: any = { ...zone };
    delete formatted_zone.id;
    delete formatted_zone.inserted_at;
    delete formatted_zone.actions;
    return formatted_zone;
  });
  let formatted_actions = actions.map((action) => {
    let formatted_action: any = { ...action };
    delete formatted_action.id;
    delete formatted_action.zone_id;
    delete formatted_action.inserted_at;
    return formatted_action;
  });
  const supabase = await newServerClient();
  // insert data and return the response
  const { data, error } = await supabase
    .from("zones")
    .insert(formatted_zones)
    .select("*");
  if (error) {
    let msg = `Error creating zones: ${error.message}`;
    return {
      status: EnumWebResponse.ERROR,
      msg: msg,
      data: null,
    };
  }
  // get zone id
  let zone_ids: number[] = data.map((zone) => {
    return zone.id;
  });
  // if zone ids length is zero, return error
  if (zone_ids.length === 0) {
    return {
      status: EnumWebResponse.ERROR,
      msg: "No zones created",
      data: null,
    };
  }
  // add zone id to formatted actions
  formatted_actions.forEach((action) => {
    action.zone_id = zone_ids[0];
  });
  // insert actions
  const { data: data_actions, error: error_actions } = await supabase
    .from("actions")
    .insert(formatted_actions)
    .select("*");
  if (error_actions) {
    let msg = `Error creating actions: ${error_actions.message}`;
    return {
      status: EnumWebResponse.ERROR,
      msg: msg,
      data: null,
    };
  }
  // merge data and data_actions
  const merged_data = data.map((zone) => {
    return {
      ...zone,
      actions: data_actions.filter((action) => action.zone_id === zone.id),
    };
  });
  return IWebResponse.success(merged_data).to_compatible();
}
