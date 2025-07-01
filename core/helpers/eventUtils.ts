/**
 * Gets the media URL for an event (signed URL if file_path exists, otherwise media_url)
 * @param event - Event object that may have file_path or media_url
 * @returns string | null - The media URL or null if none available
 */
export function getEventMediaUrl(event: any): string | null {
  return event.media_url || null;
}

/**
 * Checks if an event has any media URL available
 * @param event - Event object
 * @returns boolean - True if event has any media URL
 */
export function hasEventMedia(event: any): boolean {
  return !!(event.media_url || event.file_path);
}
