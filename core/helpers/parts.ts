import { Database } from "../types/supabase";
import { IPart, PartInsert } from "../types/db";
import { IWebResponse, IWebResponseCompatible } from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";

export async function get_parts_by_device_id(
  client: SupabaseClient<Database>,
  device_id: number,
): Promise<IWebResponseCompatible<IPart[]>> {
  const { data, error } = await client
    .from("parts")
    .select("*")
    .eq("device_id", device_id)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IPart[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart[]>(
      "No parts found for device",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_part_by_id(
  client: SupabaseClient<Database>,
  part_id: number,
): Promise<IWebResponseCompatible<IPart | null>> {
  const { data, error } = await client
    .from("parts")
    .select("*")
    .eq("id", part_id)
    .single();

  if (error) {
    return IWebResponse.error<IPart | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart | null>("Part not found").to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_parts_by_serial_number(
  client: SupabaseClient<Database>,
  serial_number: string,
): Promise<IWebResponseCompatible<IPart[]>> {
  const { data, error } = await client
    .from("parts")
    .select("*")
    .eq("serial_number", serial_number)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IPart[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart[]>(
      `No parts found with serial number: ${serial_number}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_parts_by_product_number(
  client: SupabaseClient<Database>,
  product_number: string,
): Promise<IWebResponseCompatible<IPart[]>> {
  const { data, error } = await client
    .from("parts")
    .select("*")
    .eq("product_number", product_number)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IPart[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart[]>(
      `No parts found with product number: ${product_number}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_parts_by_status(
  client: SupabaseClient<Database>,
  status: Database["public"]["Enums"]["component_status"],
): Promise<IWebResponseCompatible<IPart[]>> {
  const { data, error } = await client
    .from("parts")
    .select("*")
    .eq("status", status)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IPart[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart[]>(
      `No parts found with status: ${status}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function create_part(
  client: SupabaseClient<Database>,
  newPart: PartInsert,
): Promise<IWebResponseCompatible<IPart | null>> {
  // Validate required fields
  if (!newPart.device_id) {
    return IWebResponse.error<IPart | null>(
      "Device ID is required",
    ).to_compatible();
  }

  if (!newPart.serial_number) {
    return IWebResponse.error<IPart | null>(
      "Serial number is required",
    ).to_compatible();
  }

  const { data, error } = await client
    .from("parts")
    .insert([newPart])
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IPart | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart | null>(
      "Failed to create part",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function update_part(
  client: SupabaseClient<Database>,
  part_id: number,
  updatedPart: Partial<PartInsert>,
): Promise<IWebResponseCompatible<IPart | null>> {
  // Remove fields that shouldn't be updated
  const updateData = { ...updatedPart };
  delete (updateData as any).id;
  delete (updateData as any).created_at;

  const { data, error } = await client
    .from("parts")
    .update(updateData)
    .eq("id", part_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IPart | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart | null>(
      "Part not found or update failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function delete_part(
  client: SupabaseClient<Database>,
  part_id: number,
): Promise<IWebResponseCompatible<IPart | null>> {
  const { data, error } = await client
    .from("parts")
    .delete()
    .eq("id", part_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IPart | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart | null>(
      "Part not found or deletion failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function update_part_status(
  client: SupabaseClient<Database>,
  part_id: number,
  status: Database["public"]["Enums"]["component_status"],
): Promise<IWebResponseCompatible<IPart | null>> {
  const { data, error } = await client
    .from("parts")
    .update({ status })
    .eq("id", part_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IPart | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart | null>(
      "Part not found or status update failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_parts_by_certificate_id(
  client: SupabaseClient<Database>,
  certificate_id: number,
): Promise<IWebResponseCompatible<IPart[]>> {
  const { data, error } = await client
    .from("parts")
    .select("*")
    .eq("certificate_id", certificate_id)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IPart[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart[]>(
      `No parts found with certificate ID: ${certificate_id}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_parts_by_herd_id(
  client: SupabaseClient<Database>,
  herd_id: number,
): Promise<IWebResponseCompatible<IPart[]>> {
  const { data, error } = await client
    .from("parts")
    .select(
      `
      *,
      devices!parts_device_id_fkey(herd_id)
    `,
    )
    .eq("devices.herd_id", herd_id)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IPart[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart[]>(
      `No parts found for herd: ${herd_id}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}
