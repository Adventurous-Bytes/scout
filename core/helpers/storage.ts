"use server";

import { newServerClient } from "../supabase/server";
import { SupabaseClient } from "@supabase/supabase-js";
import { Database } from "../types/supabase";
import { BUCKET_NAME_SCOUT } from "../constants/db";

/**
 * Generates a signed URL for a file in Supabase storage
 * @param filePath - The path to the file in storage (e.g., "events/123/image.jpg")
 * @param expiresIn - Number of seconds until the URL expires (default: 3600 = 1 hour)
 * @param supabaseClient - Optional Supabase client (will create new one if not provided)
 * @returns Promise<string | null> - The signed URL or null if error
 */
export async function generateSignedUrl(
  filePath: string,
  expiresIn: number = 3600,
  supabaseClient?: SupabaseClient<Database>
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

/**
 * Generates signed URLs for multiple events and sets them as media_url
 * @param events - Array of events that may have file_path
 * @param supabaseClient - Optional Supabase client (will create new one if not provided)
 * @returns Promise<Array> - Events with signed URLs set as media_url
 */
export async function addSignedUrlsToEvents(
  events: any[],
  supabaseClient?: SupabaseClient<Database>
): Promise<any[]> {
  const eventsWithSignedUrls = await Promise.all(
    events.map(async (event) => {
      // If event has a file_path, generate a signed URL and set it as media_url
      if (event.file_path) {
        const signedUrl = await generateSignedUrl(
          event.file_path,
          3600,
          supabaseClient
        );
        return {
          ...event,
          media_url: signedUrl || event.media_url, // Fall back to existing media_url if signed URL fails
        };
      }
      // If no file_path, keep existing media_url
      return event;
    })
  );

  return eventsWithSignedUrls;
}

/**
 * Generates a signed URL for a single event and sets it as media_url
 * @param event - Event object that may have file_path
 * @param supabaseClient - Optional Supabase client (will create new one if not provided)
 * @returns Promise<Object> - Event with signed URL set as media_url
 */
export async function addSignedUrlToEvent(
  event: any,
  supabaseClient?: SupabaseClient<Database>
): Promise<any> {
  // If event has a file_path, generate a signed URL and set it as media_url
  if (event.file_path) {
    const signedUrl = await generateSignedUrl(
      event.file_path,
      3600,
      supabaseClient
    );
    return {
      ...event,
      media_url: signedUrl || event.media_url, // Fall back to existing media_url if signed URL fails
    };
  }
  // If no file_path, keep existing media_url
  return event;
}
