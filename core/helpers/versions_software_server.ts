"use server";

import { newServerClient } from "../supabase/server";
import {
  IVersionsSoftware,
  IVersionsSoftwareWithBuildUrl,
} from "../types/db";
import { IWebResponse, IWebResponseCompatible } from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";
import { generateSignedUrlsBatch } from "./storage";

/** Resolves path_build_artifact to signed URLs. Paths must be full storage paths (e.g. software_build_artifacts/...). */
async function attachBuildArtifactUrls(
  data: IVersionsSoftware[],
  client?: SupabaseClient,
): Promise<IVersionsSoftwareWithBuildUrl[]> {
  const uniquePaths = Array.from(
    new Set(
      data
        .map((v) => v.path_build_artifact)
        .filter((path): path is string => !!path),
    ),
  );

  if (uniquePaths.length === 0) {
    return data.map((v) => ({ ...v, build_artifact_url: null }));
  }

  const signedUrls = await generateSignedUrlsBatch(
    uniquePaths,
    undefined,
    client,
  );
  const urlMap = new Map<string, string | null>();
  uniquePaths.forEach((path, index) => {
    urlMap.set(path, signedUrls[index]);
  });

  return data.map((v) => ({
    ...v,
    build_artifact_url: v.path_build_artifact
      ? urlMap.get(v.path_build_artifact) ?? null
      : null,
  }));
}

export async function server_get_versions_software(
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IVersionsSoftwareWithBuildUrl[]>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase
    .from("versions_software")
    .select("*")
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IVersionsSoftwareWithBuildUrl[]>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IVersionsSoftwareWithBuildUrl[]>(
      "No software versions found",
    ).to_compatible();
  }

  const withUrls = await attachBuildArtifactUrls(data, client);
  return IWebResponse.success(withUrls).to_compatible();
}

export async function server_get_versions_software_by_system(
  system: string,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IVersionsSoftwareWithBuildUrl[]>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase
    .from("versions_software")
    .select("*")
    .eq("system", system)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IVersionsSoftwareWithBuildUrl[]>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IVersionsSoftwareWithBuildUrl[]>(
      `No software versions found for system: ${system}`,
    ).to_compatible();
  }

  const withUrls = await attachBuildArtifactUrls(data, client);
  return IWebResponse.success(withUrls).to_compatible();
}

export async function server_get_versions_software_by_created_by(
  user_id: string,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IVersionsSoftwareWithBuildUrl[]>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase
    .from("versions_software")
    .select("*")
    .eq("created_by", user_id)
    .order("created_at", { ascending: false });

  if (error) {
    return IWebResponse.error<IVersionsSoftwareWithBuildUrl[]>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IVersionsSoftwareWithBuildUrl[]>(
      "No software versions found for user",
    ).to_compatible();
  }

  const withUrls = await attachBuildArtifactUrls(data, client);
  return IWebResponse.success(withUrls).to_compatible();
}
