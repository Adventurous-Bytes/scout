import { Database } from "../types/supabase";
import { IPin, PinInsert } from "../types/db";
import { IWebResponse, IWebResponseCompatible } from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";

export async function get_pins_for_herd(
  client: SupabaseClient<Database>,
  herd_id: number,
): Promise<IWebResponseCompatible<IPin[]>> {
  // Call get_pins_for_herd with rpc - returns pins_pretty_location with extracted coordinates
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

  // Raw table data - no coordinate extraction (use get_pins_for_herd for coordinates)
  return IWebResponse.success(data as any).to_compatible();
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

  // Raw table data - coordinates extracted by RPC function or realtime broadcasts
  return IWebResponse.success(data as any).to_compatible();
}

export async function create_pin_with_coordinates(
  client: SupabaseClient<Database>,
  latitude: number,
  longitude: number,
  pinData: Omit<PinInsert, "location">,
): Promise<IWebResponseCompatible<IPin | null>> {
  // Create pin with PostGIS Point from lat/lng coordinates
  const { data, error } = await client
    .from("pins")
    .insert([
      {
        ...pinData,
        // Use PostGIS ST_MakePoint to create geography from coordinates
        // Note: PostGIS Point format is (longitude, latitude)
        location: `SRID=4326;POINT(${longitude} ${latitude})`,
      },
    ])
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

  return IWebResponse.success(data as any).to_compatible();
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

  // Raw table data - coordinates extracted by RPC function or realtime broadcasts
  return IWebResponse.success(data as any).to_compatible();
}

export async function update_pin_location(
  client: SupabaseClient<Database>,
  pin_id: number,
  latitude: number,
  longitude: number,
  updatedPin?: Partial<Omit<PinInsert, "location">>,
): Promise<IWebResponseCompatible<IPin | null>> {
  const updateData = {
    ...updatedPin,
    // Use PostGIS format to update location
    location: `SRID=4326;POINT(${longitude} ${latitude})`,
  };

  // Remove fields that shouldn't be updated
  delete (updateData as any).id;
  delete (updateData as any).created_at;
  delete (updateData as any).created_by;

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

  return IWebResponse.success(data as any).to_compatible();
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

  // Raw table data - coordinates extracted by realtime broadcasts
  return IWebResponse.success(data as any).to_compatible();
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

  // Raw table data without extracted coordinates - use get_pins_for_herd for coordinates
  return IWebResponse.success(data as any[]).to_compatible();
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

  // Raw table data without extracted coordinates - use get_pins_for_herd for coordinates
  return IWebResponse.success(data as any[]).to_compatible();
}
