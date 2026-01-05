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
    .is("deleted_at", null)
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
    .is("deleted_at", null)
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
    .is("deleted_at", null)
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
    .is("deleted_at", null)
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
    .is("deleted_at", null)
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

  if (!newPart.product_number) {
    return IWebResponse.error<IPart | null>(
      "Product number is required",
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
  // Soft delete by setting deleted_at timestamp
  const { data, error } = await client
    .from("parts")
    .update({ deleted_at: new Date().toISOString() })
    .eq("id", part_id)
    .is("deleted_at", null)
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
    .is("deleted_at", null)
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
    .is("deleted_at", null)
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
    .is("deleted_at", null)
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

export async function restore_part(
  client: SupabaseClient<Database>,
  part_id: number,
): Promise<IWebResponseCompatible<IPart | null>> {
  // Restore soft deleted part by setting deleted_at to null
  const { data, error } = await client
    .from("parts")
    .update({ deleted_at: null })
    .eq("id", part_id)
    .not("deleted_at", "is", null)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IPart | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart | null>(
      "Part not found or restore failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function hard_delete_part(
  client: SupabaseClient<Database>,
  part_id: number,
): Promise<IWebResponseCompatible<IPart | null>> {
  // Permanently delete the part (only use for already soft-deleted parts)
  const { data, error } = await client
    .from("parts")
    .delete()
    .eq("id", part_id)
    .not("deleted_at", "is", null)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IPart | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart | null>(
      "Part not found or permanent deletion failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_deleted_parts_by_device_id(
  client: SupabaseClient<Database>,
  device_id: number,
): Promise<IWebResponseCompatible<IPart[]>> {
  const { data, error } = await client
    .from("parts")
    .select("*")
    .eq("device_id", device_id)
    .not("deleted_at", "is", null)
    .order("deleted_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IPart[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart[]>(
      "No deleted parts found for device",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_parts_by_product_and_serial(
  client: SupabaseClient<Database>,
  product_number: string,
  serial_number: string,
): Promise<IWebResponseCompatible<IPart | null>> {
  // Get part by the composite unique constraint
  const { data, error } = await client
    .from("parts")
    .select("*")
    .eq("product_number", product_number)
    .eq("serial_number", serial_number)
    .is("deleted_at", null)
    .single();

  if (error) {
    return IWebResponse.error<IPart | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IPart | null>(
      `No part found with product number: ${product_number} and serial number: ${serial_number}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}
