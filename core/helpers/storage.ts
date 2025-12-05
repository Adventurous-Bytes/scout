"use server";

import { newServerClient } from "../supabase/server";
import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import {
  BUCKET_NAME_SCOUT,
  SIGNED_URL_EXPIRATION_SECONDS,
} from "../constants/db";

/**
 * Splits file path into bucket name and path. Must be at leaast one slash in your file path
 * @param filePath
 * @returns IFilePathParts | null - null if invalid file path
 */
function getBucketFromFilePath(filePath: string): string | null {
  // delete any start or end slashes/whitespace
  filePath = filePath.replace(/^\/+|\/+$/g, "");
  const parts = filePath.split("/");
  if (parts.length < 2) {
    return null;
  }
  const bucket_name = parts[0];
  return bucket_name;
}

/**
 * Extracts the short path from a file path
 * @param filePath
 * @returns string | null - null if invalid file path
 */
// for example if the input is /artifacts/10/52/test.mp4 - the output shld be 10/52/test.mp4
function getFormattedPath(filePath: string): string | null {
  const cleaned = cleanPath(filePath);
  const parts = cleaned.split("/");
  if (parts.length < 2) {
    return null;
  }
  // Remove the first part (bucket name) and return the rest
  return parts.slice(1).join("/");
}

/** Removes leading and trailing slashes and leading/trailing whitespace */
function cleanPath(filePath: string): string {
  // delete leading/trailing slash and whitespace
  return filePath.trim().replace(/^\/+|\/+$/g, "");
}

/**
 * Generates a signed URL for a file in Supabase storage
 * @param filePath - The path to the file in storage (e.g., "events/123/image.jpg")
 * @param expiresIn - Number of seconds until the URL expires (default: 12 hours)
 * @param supabaseClient - Optional Supabase client (will create new one if not provided)
 * @returns Promise<string | null> - The signed URL or null if error
 */
export async function generateSignedUrl(
  filePath: string,
  expiresIn: number = SIGNED_URL_EXPIRATION_SECONDS,
  supabaseClient?: SupabaseClient<Database>,
): Promise<string | null> {
  try {
    const supabase = supabaseClient || (await newServerClient());
    const bucket_name = getBucketFromFilePath(filePath);
    if (!bucket_name) {
      console.error("Invalid file path:", filePath);
      return null;
    }
    const formattedPath = getFormattedPath(filePath);
    if (!formattedPath) {
      console.error("Invalid formatted path:", formattedPath);
      return null;
    }
    const { data, error } = await supabase.storage
      .from(bucket_name)
      .createSignedUrl(formattedPath, expiresIn);
    if (error) {
      console.error("Error generating signed URL:", error.message);
      return null;
    }
    return data.signedUrl;
  } catch (error) {
    console.error("Error in generateSignedUrl:", error);
    return null;
  }
}

export async function generateSignedUrlsBatch(
  filePaths: string[],
  expiresIn: number = SIGNED_URL_EXPIRATION_SECONDS,
  supabaseClient?: SupabaseClient<Database>,
): Promise<(string | null)[]> {
  try {
    const supabase = supabaseClient || (await newServerClient());

    const signedUrlPromises = filePaths.map(async (filePath) => {
      try {
        const bucket_name = getBucketFromFilePath(filePath);
        if (!bucket_name) {
          console.error("Invalid file path:", filePath);
          return null;
        }
        const formattedPath = getFormattedPath(filePath);
        if (!formattedPath) {
          console.error("Invalid formatted path:", formattedPath);
          return null;
        }
        const { data, error } = await supabase.storage
          .from(bucket_name)
          .createSignedUrl(formattedPath, expiresIn);

        if (error) {
          console.warn(
            `Error generating signed URL for ${filePath}:`,
            error.message,
          );
          return null;
        }

        return data.signedUrl;
      } catch (error) {
        console.warn(`Exception generating signed URL for ${filePath}:`, error);
        return null;
      }
    });

    const results = await Promise.all(signedUrlPromises);
    return results;
  } catch (error) {
    console.error("Error in generateSignedUrlsBatch:", error);
    return new Array(filePaths.length).fill(null);
  }
}
