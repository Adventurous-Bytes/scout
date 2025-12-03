import { Database } from "../types/supabase";
import { IComponent, ComponentInsert } from "../types/db";
import { IWebResponse, IWebResponseCompatible } from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";

export async function get_components_by_device_id(
  client: SupabaseClient<Database>,
  device_id: number,
): Promise<IWebResponseCompatible<IComponent[]>> {
  const { data, error } = await client
    .from("components")
    .select("*")
    .eq("device_id", device_id)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IComponent[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent[]>(
      "No components found for device",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_component_by_id(
  client: SupabaseClient<Database>,
  component_id: number,
): Promise<IWebResponseCompatible<IComponent | null>> {
  const { data, error } = await client
    .from("components")
    .select("*")
    .eq("id", component_id)
    .single();

  if (error) {
    return IWebResponse.error<IComponent | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent | null>(
      "Component not found",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_components_by_serial_number(
  client: SupabaseClient<Database>,
  serial_number: string,
): Promise<IWebResponseCompatible<IComponent[]>> {
  const { data, error } = await client
    .from("components")
    .select("*")
    .eq("serial_number", serial_number)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IComponent[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent[]>(
      `No components found with serial number: ${serial_number}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_components_by_product_number(
  client: SupabaseClient<Database>,
  product_number: string,
): Promise<IWebResponseCompatible<IComponent[]>> {
  const { data, error } = await client
    .from("components")
    .select("*")
    .eq("product_number", product_number)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IComponent[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent[]>(
      `No components found with product number: ${product_number}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_components_by_status(
  client: SupabaseClient<Database>,
  status: Database["public"]["Enums"]["component_status"],
): Promise<IWebResponseCompatible<IComponent[]>> {
  const { data, error } = await client
    .from("components")
    .select("*")
    .eq("status", status)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IComponent[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent[]>(
      `No components found with status: ${status}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function create_component(
  client: SupabaseClient<Database>,
  newComponent: ComponentInsert,
): Promise<IWebResponseCompatible<IComponent | null>> {
  // Validate required fields
  if (!newComponent.device_id) {
    return IWebResponse.error<IComponent | null>(
      "Device ID is required",
    ).to_compatible();
  }

  if (!newComponent.serial_number) {
    return IWebResponse.error<IComponent | null>(
      "Serial number is required",
    ).to_compatible();
  }

  const { data, error } = await client
    .from("components")
    .insert([newComponent])
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IComponent | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent | null>(
      "Failed to create component",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function update_component(
  client: SupabaseClient<Database>,
  component_id: number,
  updatedComponent: Partial<ComponentInsert>,
): Promise<IWebResponseCompatible<IComponent | null>> {
  // Remove fields that shouldn't be updated
  const updateData = { ...updatedComponent };
  delete (updateData as any).id;
  delete (updateData as any).created_at;

  const { data, error } = await client
    .from("components")
    .update(updateData)
    .eq("id", component_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IComponent | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent | null>(
      "Component not found or update failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function delete_component(
  client: SupabaseClient<Database>,
  component_id: number,
): Promise<IWebResponseCompatible<IComponent | null>> {
  const { data, error } = await client
    .from("components")
    .delete()
    .eq("id", component_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IComponent | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent | null>(
      "Component not found or deletion failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function update_component_status(
  client: SupabaseClient<Database>,
  component_id: number,
  status: Database["public"]["Enums"]["component_status"],
): Promise<IWebResponseCompatible<IComponent | null>> {
  const { data, error } = await client
    .from("components")
    .update({ status })
    .eq("id", component_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IComponent | null>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent | null>(
      "Component not found or status update failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_components_by_certificate_id(
  client: SupabaseClient<Database>,
  certificate_id: number,
): Promise<IWebResponseCompatible<IComponent[]>> {
  const { data, error } = await client
    .from("components")
    .select("*")
    .eq("certificate_id", certificate_id)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IComponent[]>(error.message).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IComponent[]>(
      `No components found with certificate ID: ${certificate_id}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}
