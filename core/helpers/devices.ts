"use server";

import { newServerClient } from "../supabase/server";
import { IDevice } from "../types/db";
import { IWebResponse, IWebResponseCompatible } from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";

export async function get_devices_by_herd(
  herd_id: number,
  client: SupabaseClient
): Promise<IWebResponseCompatible<IDevice[]>> {
  // call get_devices_for_herd with rpc
  const { data, error } = await client.rpc("get_devices_for_herd", {
    herd_id_caller: herd_id,
  });

  if (!data) {
    return IWebResponse.error<IDevice[]>("No devices found").to_compatible();
  } else {
    let response: IWebResponse<IDevice[]> = IWebResponse.success(data);
    return response.to_compatible();
  }
}

export async function get_device_by_id(
  device_id: number,
  client?: SupabaseClient
): Promise<IWebResponseCompatible<IDevice | null>> {
  if (!client) {
    client = await newServerClient();
  }
  const { data, error } = await client.rpc("get_device_by_id", {
    device_id_caller: device_id,
  });

  if (!data) {
    return IWebResponse.error<IDevice | null>(
      "No device found"
    ).to_compatible();
  } else {
    let response: IWebResponse<IDevice> = IWebResponse.success(data);
    return response.to_compatible();
  }
}

export async function serverUpdateDevice(
  updatedDevice: IDevice
): Promise<IWebResponseCompatible<IDevice | null>> {
  // delete api keys, latitide, and longitude
  const device_formatted: any = { ...updatedDevice };
  delete device_formatted.api_keys_scout;
  const device_latitude = device_formatted.latitude;
  const device_longitude = device_formatted.longitude;
  delete device_formatted.latitude;
  delete device_formatted.longitude;
  delete device_formatted.recent_events;
  const supabase = await newServerClient();
  const { data, error } = await supabase
    .from("devices")
    .update(device_formatted)
    .match({ id: device_formatted.id })
    .select("*");
  if (error) {
    return IWebResponse.error<IDevice | null>(error.message).to_compatible();
  } else {
    const updatedDevice: any = {
      ...data[0],
      latitude: device_latitude,
      longitude: device_longitude,
    };
    return IWebResponse.success(updatedDevice).to_compatible();
  }
}

export async function serverCreateDevice(
  newDevice: any
): Promise<IWebResponseCompatible<IDevice | null>> {
  const supabase = await newServerClient();
  const user = await supabase.auth.getUser();
  const userId = user?.data?.user?.id;
  newDevice.created_by = userId;
  // strip id field from herd object
  const { data, error } = await supabase
    .from("devices")
    .insert([newDevice])
    .select("*");
  if (error) {
    // TODO: ALLOW PROPERTY INSTANTION OF CPMPATIBLE WEB RESPONSE
    return IWebResponse.error<IDevice | null>(error.message).to_compatible();
  } else {
    const newDevice: any = { ...data[0], latitude: 0, longitude: 0 };
    return IWebResponse.success(newDevice).to_compatible();
  }
}

export async function serverDeleteDeviceById(
  device_id: number
): Promise<IWebResponseCompatible<IDevice | null>> {
  const supabase = await newServerClient();
  const { data, error } = await supabase
    .from("devices")
    .delete()
    .match({ id: device_id })
    .select("*");
  if (error) {
    return IWebResponse.error<IDevice | null>(error.message).to_compatible();
  } else {
    const deletedDevice: any = { ...data[0], latitude: 0, longitude: 0 };
    return IWebResponse.success(deletedDevice).to_compatible();
  }
}
