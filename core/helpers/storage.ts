"use server";

import { newServerClient } from "../supabase/server";
import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import {
  BUCKET_NAME_SCOUT,
  SIGNED_URL_EXPIRATION_SECONDS,
} from "../constants/db";

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

    const { data, error } = await supabase.storage
      .from(BUCKET_NAME_SCOUT)
      .createSignedUrl(filePath, expiresIn);
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
        const { data, error } = await supabase.storage
          .from(BUCKET_NAME_SCOUT)
          .createSignedUrl(filePath, expiresIn);

        if (error) {
          console.error(
            `Error generating signed URL for ${filePath}:`,
            error.message,
          );
          return null;
        }

        return data.signedUrl;
      } catch (error) {
        console.error(
          `Exception generating signed URL for ${filePath}:`,
          error,
        );
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
