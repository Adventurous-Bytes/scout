import { Database } from "../types/supabase";
import { IPin, PinInsert } from "../types/db";
import { IWebResponse, IWebResponseCompatible } from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";

export async function get_pins_for_herd(
  client: SupabaseClient<Database>,
  herd_id: number,
): Promise<IWebResponseCompatible<IPin[]>> {
  // Call get_pins_for_herd with rpc
  const { data, error } = await client.rpc("get_pins_for_herd", {
    herd_id_caller: herd_id,
  });

  if (error) {
    return IWebResponse.error<IPin[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPin[]>("No pins found for herd").to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_pin_by_id(
  client: SupabaseClient<Database>,
  pin_id: number,
): Promise<IWebResponseCompatible<IPin | null>> {
  const { data, error } = await client
    .from("pins")
    .select("*")
    .eq("id", pin_id)
    .single();

  if (error) {
    return IWebResponse.error<IPin | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPin | null>("Pin not found").to_compatible();
  }

  // Convert to pretty location format with coordinates
  const pinWithCoords: IPin = {
    ...data,
    latitude: data.latitude ? parseFloat(data.latitude.toString()) : 0,
    longitude: data.longitude ? parseFloat(data.longitude.toString()) : 0,
  };

  return IWebResponse.success(pinWithCoords).to_compatible();
}

export async function create_pin(
  client: SupabaseClient<Database>,
  newPin: PinInsert,
): Promise<IWebResponseCompatible<IPin | null>> {
  const { data, error } = await client
    .from("pins")
    .insert([newPin])
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IPin | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPin | null>(
      "Failed to create pin",
    ).to_compatible();
  }

  // Convert to pretty location format with coordinates
  const pinWithCoords: IPin = {
    ...data,
    latitude: data.latitude ? parseFloat(data.latitude.toString()) : 0,
    longitude: data.longitude ? parseFloat(data.longitude.toString()) : 0,
  };

  return IWebResponse.success(pinWithCoords).to_compatible();
}

export async function update_pin(
  client: SupabaseClient<Database>,
  pin_id: number,
  updatedPin: Partial<PinInsert>,
): Promise<IWebResponseCompatible<IPin | null>> {
  // Remove fields that shouldn't be updated
  const updateData = { ...updatedPin };
  delete (updateData as any).id;
  delete (updateData as any).created_at;
  delete (updateData as any).created_by; // RLS handles permissions

  const { data, error } = await client
    .from("pins")
    .update(updateData)
    .eq("id", pin_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IPin | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPin | null>(
      "Pin not found or update failed",
    ).to_compatible();
  }

  // Convert to pretty location format with coordinates
  const pinWithCoords: IPin = {
    ...data,
    latitude: data.latitude ? parseFloat(data.latitude.toString()) : 0,
    longitude: data.longitude ? parseFloat(data.longitude.toString()) : 0,
  };

  return IWebResponse.success(pinWithCoords).to_compatible();
}

export async function delete_pin(
  client: SupabaseClient<Database>,
  pin_id: number,
): Promise<IWebResponseCompatible<IPin | null>> {
  const { data, error } = await client
    .from("pins")
    .delete()
    .eq("id", pin_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IPin | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPin | null>(
      "Pin not found or deletion failed",
    ).to_compatible();
  }

  // Convert to pretty location format with coordinates
  const pinWithCoords: IPin = {
    ...data,
    latitude: data.latitude ? parseFloat(data.latitude.toString()) : 0,
    longitude: data.longitude ? parseFloat(data.longitude.toString()) : 0,
  };

  return IWebResponse.success(pinWithCoords).to_compatible();
}

export async function get_pins_by_created_by(
  client: SupabaseClient<Database>,
  user_id: string,
): Promise<IWebResponseCompatible<IPin[]>> {
  const { data, error } = await client
    .from("pins")
    .select("*")
    .eq("created_by", user_id)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IPin[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPin[]>("No pins found for user").to_compatible();
  }

  // Convert to pretty location format with coordinates
  const pinsWithCoords: IPin[] = data.map((pin) => ({
    ...pin,
    latitude: pin.latitude ? parseFloat(pin.latitude.toString()) : 0,
    longitude: pin.longitude ? parseFloat(pin.longitude.toString()) : 0,
  }));

  return IWebResponse.success(pinsWithCoords).to_compatible();
}

export async function get_pins_by_color(
  client: SupabaseClient<Database>,
  herd_id: number,
  color: string,
): Promise<IWebResponseCompatible<IPin[]>> {
  const { data, error } = await client
    .from("pins")
    .select("*")
    .eq("herd_id", herd_id)
    .eq("color", color)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IPin[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPin[]>(
      `No pins found with color: ${color}`,
    ).to_compatible();
  }

  // Convert to pretty location format with coordinates
  const pinsWithCoords: IPin[] = data.map((pin) => ({
    ...pin,
    latitude: pin.latitude ? parseFloat(pin.latitude.toString()) : 0,
    longitude: pin.longitude ? parseFloat(pin.longitude.toString()) : 0,
  }));

  return IWebResponse.success(pinsWithCoords).to_compatible();
}
