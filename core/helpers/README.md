# Scout Core Helpers

## Storage and Signed URLs

This module provides functionality for handling file storage and signed URL generation for media files.

### Key Features

- **Signed URL Generation**: Generate secure, time-limited URLs for accessing media files
- **Backward Compatibility**: Support for both new signed URLs and legacy public URLs
- **Seamless Integration**: Signed URLs are automatically set as `media_url` for easy use
- **Efficient Client Usage**: Accepts existing Supabase client to avoid creating multiple instances

### Files

- **`storage.ts`**: Server-side functions for generating signed URLs (requires "use server")
- **`eventUtils.ts`**: Client-side utility functions for working with event media URLs

### Storage Functions (Server-side)

#### `generateSignedUrl(filePath, expiresIn, supabaseClient?)`

Generates a signed URL for a file in Supabase storage.

```typescript
// With existing client
const signedUrl = await generateSignedUrl(
  "events/123/image.jpg",
  3600,
  supabaseClient
);

// Without existing client (creates new one)
const signedUrl = await generateSignedUrl("events/123/image.jpg", 3600);
```

#### `addSignedUrlsToEvents(events, supabaseClient?)`

Adds signed URLs to an array of events, setting them as `media_url`.

```typescript
// With existing client
const eventsWithUrls = await addSignedUrlsToEvents(events, supabaseClient);

// Without existing client (creates new one)
const eventsWithUrls = await addSignedUrlsToEvents(events);
```

#### `addSignedUrlToEvent(event, supabaseClient?)`

Adds a signed URL to a single event, setting it as `media_url`.

```typescript
// With existing client
const eventWithUrl = await addSignedUrlToEvent(event, supabaseClient);

// Without existing client (creates new one)
const eventWithUrl = await addSignedUrlToEvent(event);
```

### Event Utility Functions (Client-side)

#### `getEventMediaUrl(event)`

Gets the media URL for an event (now simply returns `event.media_url`).

```typescript
const mediaUrl = getEventMediaUrl(event);
```

#### `hasEventMedia(event)`

Checks if an event has any media URL available.

```typescript
const hasMedia = hasEventMedia(event);
```

### Usage in Components

```typescript
import { getEventMediaUrl, hasEventMedia } from "@adventurelabs/scout-core";

function EventMedia({ event }) {
  const mediaUrl = getEventMediaUrl(event);

  if (!hasEventMedia(event)) {
    return <div>No media available</div>;
  }

  return <img src={mediaUrl} alt="Event media" />;
}
```

## Heartbeats

This module provides functionality for managing device heartbeats and checking device online status.

### Key Features

- **Last Heartbeat Retrieval**: Get the most recent heartbeat for a device
- **Device Status Checking**: Determine if a device is online based on heartbeat recency
- **Batch Heartbeat Retrieval**: Get multiple heartbeats for a device with optional limits
- **Flexible Thresholds**: Configurable offline detection thresholds

### Files

- **`heartbeats.ts`**: Server-side functions for heartbeat management (requires "use server")

### Heartbeat Functions (Server-side)

#### `server_get_last_heartbeat_by_device(device_id)`

Gets the most recent heartbeat for a specific device.

```typescript
const response = await server_get_last_heartbeat_by_device(123);
if (response.status === EnumWebResponse.SUCCESS && response.data) {
  console.log(`Last heartbeat: ${response.data.timestamp}`);
} else {
  console.log("No heartbeats found for device");
}
```

#### `server_get_heartbeats_by_device(device_id, limit?)`

Gets all heartbeats for a device, optionally limited to a specific count.

```typescript
// Get all heartbeats
const response = await server_get_heartbeats_by_device(123);

// Get last 10 heartbeats
const response = await server_get_heartbeats_by_device(123, 10);
```

#### `server_check_device_online_status(device_id, offline_threshold_minutes?)`

Checks if a device is considered online based on heartbeat recency (default threshold: 5 minutes).

```typescript
const response = await server_check_device_online_status(123, 10);
if (response.status === EnumWebResponse.SUCCESS && response.data) {
  const { is_online, last_heartbeat, minutes_since_last_heartbeat } = response.data;
  console.log(`Device is ${is_online ? 'online' : 'offline'}`);
  if (minutes_since_last_heartbeat !== null) {
    console.log(`Last seen ${minutes_since_last_heartbeat} minutes ago`);
  }
}
```

### Database Schema

Events now have two URL-related fields:

- `file_path`: The storage path for generating signed URLs
- `media_url`: The URL for accessing media (signed URL when file_path exists, legacy public URL otherwise)

### How It Works

1. **Event Creation**: Files are uploaded and `file_path` is stored (no `media_url` initially)
2. **Event Fetching**: When events are retrieved, signed URLs are generated and set as `media_url`
3. **URL Access**: Components simply use `event.media_url` as before
4. **Fallback**: If signed URL generation fails, existing `media_url` is preserved

### Performance Optimization

The storage functions accept an optional `supabaseClient` parameter to reuse existing client instances:

```typescript
// Efficient: Reuse existing client
const supabase = await newServerClient();
const events = await fetchEvents(supabase);
const eventsWithUrls = await addSignedUrlsToEvents(events, supabase);

// Less efficient: Creates new client for each operation
const events = await fetchEvents();
const eventsWithUrls = await addSignedUrlsToEvents(events); // Creates new client
```

### Migration Strategy

1. **Phase 1**: Store `file_path` for new events (no `media_url`)
2. **Phase 2**: Generate signed URLs on fetch and set as `media_url`
3. **Phase 3**: All events use signed URLs seamlessly

### Security Benefits

- **Time-limited access**: URLs expire after a configurable time
- **Secure access**: URLs are cryptographically signed
- **No public exposure**: Files are not publicly accessible without signed URLs
- **Seamless integration**: No changes needed in existing components
