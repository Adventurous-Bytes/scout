"use server";

import { newServerClient } from "../supabase/server";
import { IArtifact, IArtifactWithMediaUrl } from "../types/db";
import { IWebResponse, IWebResponseCompatible } from "../types/requests";
import { SupabaseClient } from "@supabase/supabase-js";
import { generateSignedUrlsBatch, generateSignedUrl } from "./storage";

export async function server_get_artifacts_by_herd(
  herd_id: number,
  limit: number = 50,
  offset: number = 0,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IArtifactWithMediaUrl[]>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase.rpc("get_artifacts_for_herd", {
    herd_id_caller: herd_id,
    limit_caller: limit,
    offset_caller: offset,
  });

  if (error) {
    return IWebResponse.error<IArtifactWithMediaUrl[]>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IArtifactWithMediaUrl[]>(
      "No artifacts found for herd",
    ).to_compatible();
  }

  // Generate signed URLs for artifacts
  const uniqueFilePaths = Array.from(
    new Set(
      (data as IArtifact[])
        .map((artifact: IArtifact) => artifact.file_path)
        .filter((path): path is string => !!path),
    ),
  );

  if (uniqueFilePaths.length === 0) {
    return IWebResponse.success(
      data as IArtifactWithMediaUrl[],
    ).to_compatible();
  }

  const signedUrls = await generateSignedUrlsBatch(
    uniqueFilePaths,
    undefined,
    client,
  );
  const urlMap = new Map<string, string | null>();

  uniqueFilePaths.forEach((path, index) => {
    urlMap.set(path, signedUrls[index]);
  });

  const artifactsWithUrls: IArtifactWithMediaUrl[] = (data as IArtifact[]).map(
    (artifact: IArtifact) => ({
      ...artifact,
      media_url: artifact.file_path
        ? urlMap.get(artifact.file_path) || null
        : null,
    }),
  );

  return IWebResponse.success(artifactsWithUrls).to_compatible();
}

export async function server_get_artifacts_by_device_id(
  device_id: number,
  limit: number = 50,
  offset: number = 0,
  client?: SupabaseClient,
): Promise<IWebResponseCompatible<IArtifactWithMediaUrl[]>> {
  const supabase = client || (await newServerClient());

  const { data, error } = await supabase.rpc("get_artifacts_for_device", {
    device_id_caller: device_id,
    limit_caller: limit,
    offset_caller: offset,
  });

  if (error) {
    return IWebResponse.error<IArtifactWithMediaUrl[]>(
      error.message,
    ).to_compatible();
  }

  if (!data) {
    return IWebResponse.error<IArtifactWithMediaUrl[]>(
      "No artifacts found for device",
    ).to_compatible();
  }

  // Generate signed URLs for artifacts
  const uniqueFilePaths = Array.from(
    new Set(
      (data as IArtifact[])
        .map((artifact: IArtifact) => artifact.file_path)
        .filter((path): path is string => !!path),
    ),
  );

  if (uniqueFilePaths.length === 0) {
    return IWebResponse.success(
      data as IArtifactWithMediaUrl[],
    ).to_compatible();
  }

  const signedUrls = await generateSignedUrlsBatch(
    uniqueFilePaths,
    undefined,
    client,
  );
  const urlMap = new Map<string, string | null>();

  uniqueFilePaths.forEach((path, index) => {
    urlMap.set(path, signedUrls[index]);
  });

  const artifactsWithUrls: IArtifactWithMediaUrl[] = (data as IArtifact[]).map(
    (artifact: IArtifact) => ({
      ...artifact,
      media_url: artifact.file_path
        ? urlMap.get(artifact.file_path) || null
        : null,
    }),
  );

  return IWebResponse.success(artifactsWithUrls).to_compatible();
}

export async function server_get_total_artifacts_by_herd(
  herd_id: number,
): Promise<IWebResponseCompatible<number>> {
  const supabase = await newServerClient();

  const { data, error } = await supabase.rpc("get_total_artifacts_for_herd", {
    herd_id_caller: herd_id,
  });

  if (error) {
    return IWebResponse.error<number>(error.message).to_compatible();
  }

  return IWebResponse.success(data || 0).to_compatible();
}

export async function server_get_artifacts_by_device_ids_batch(
  device_ids: number[],
  limit_per_device: number = 10,
  client?: SupabaseClient,
): Promise<{ [device_id: number]: IArtifactWithMediaUrl[] }> {
  const supabase = client || (await newServerClient());

  if (device_ids.length === 0) {
    return {};
  }

  const { data, error } = await supabase.rpc(
    "get_artifacts_for_devices_batch",
    {
      device_ids: device_ids,
      limit_per_device: limit_per_device,
    },
  );

  if (error || !data) {
    console.warn("Error fetching artifacts batch:", error?.message);
    return {};
  }

  // Generate signed URLs for artifacts
  const uniqueFilePaths = Array.from(
    new Set(
      (data as IArtifact[])
        .map((artifact: IArtifact) => artifact.file_path)
        .filter((path): path is string => !!path),
    ),
  );

  let artifactsWithUrls: IArtifactWithMediaUrl[] =
    data as IArtifactWithMediaUrl[];

  if (uniqueFilePaths.length > 0) {
    const signedUrls = await generateSignedUrlsBatch(
      uniqueFilePaths,
      undefined,
      client,
    );
    const urlMap = new Map<string, string | null>();

    uniqueFilePaths.forEach((path, index) => {
      urlMap.set(path, signedUrls[index]);
    });

    artifactsWithUrls = (data as IArtifact[]).map((artifact: IArtifact) => ({
      ...artifact,
      media_url: artifact.file_path
        ? urlMap.get(artifact.file_path) || null
        : null,
    }));
  }

  // Group artifacts by device_id
  const artifactsByDevice: { [device_id: number]: IArtifactWithMediaUrl[] } =
    {};

  artifactsWithUrls.forEach((artifact) => {
    if (!artifactsByDevice[artifact.device_id]) {
      artifactsByDevice[artifact.device_id] = [];
    }
    artifactsByDevice[artifact.device_id].push(artifact);
  });

  return artifactsByDevice;
}
