import { Database } from "../types/supabase";
import { IVersionsSoftware, VersionsSoftwareInsert } from "../types/db";
import { IWebResponse, IWebResponseCompatible } from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";

export async function get_versions_software(
  client: SupabaseClient<Database>,
): Promise<IWebResponseCompatible<IVersionsSoftware[]>> {
  const { data, error } = await client
    .from("versions_software")
    .select("*")
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IVersionsSoftware[]>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IVersionsSoftware[]>(
      "No software versions found",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_versions_software_by_system(
  client: SupabaseClient<Database>,
  system: string,
): Promise<IWebResponseCompatible<IVersionsSoftware[]>> {
  const { data, error } = await client
    .from("versions_software")
    .select("*")
    .eq("system", system)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IVersionsSoftware[]>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IVersionsSoftware[]>(
      `No software versions found for system: ${system}`,
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function create_version_software(
  client: SupabaseClient<Database>,
  newVersionSoftware: VersionsSoftwareInsert,
): Promise<IWebResponseCompatible<IVersionsSoftware | null>> {
  const { data, error } = await client
    .from("versions_software")
    .insert([newVersionSoftware])
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IVersionsSoftware | null>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IVersionsSoftware | null>(
      "Failed to create software version",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function update_version_software(
  client: SupabaseClient<Database>,
  version_id: number,
  updatedVersionSoftware: Partial<VersionsSoftwareInsert>,
): Promise<IWebResponseCompatible<IVersionsSoftware | null>> {
  // Remove fields that shouldn't be updated
  const updateData = { ...updatedVersionSoftware };
  delete (updateData as any).id;
  delete (updateData as any).created_at;
  delete (updateData as any).created_by; // Only original creator can modify due to RLS

  const { data, error } = await client
    .from("versions_software")
    .update(updateData)
    .eq("id", version_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IVersionsSoftware | null>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IVersionsSoftware | null>(
      "Software version not found or update failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function delete_version_software(
  client: SupabaseClient<Database>,
  version_id: number,
): Promise<IWebResponseCompatible<IVersionsSoftware | null>> {
  const { data, error } = await client
    .from("versions_software")
    .delete()
    .eq("id", version_id)
    .select("*")
    .single();

  if (error) {
    return IWebResponse.error<IVersionsSoftware | null>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IVersionsSoftware | null>(
      "Software version not found or deletion failed",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}

export async function get_versions_software_by_created_by(
  client: SupabaseClient<Database>,
  user_id: string,
): Promise<IWebResponseCompatible<IVersionsSoftware[]>> {
  const { data, error } = await client
    .from("versions_software")
    .select("*")
    .eq("created_by", user_id)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IVersionsSoftware[]>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IVersionsSoftware[]>(
      "No software versions found for user",
    ).to_compatible();
  }

  return IWebResponse.success(data).to_compatible();
}
