"use server";

// add tag to db

import { newServerClient } from "../supabase/server";
import { IEventWithTags, ITag } from "../types/db";
import {
  EnumWebResponse,
  IWebResponse,
  IWebResponseCompatible,
} from "../types/requests";
import { addSignedUrlsToEvents, addSignedUrlToEvent } from "./storage";

// Test function to verify individual event loading works
export async function test_event_loading(device_id: number): Promise<boolean> {
  try {
    console.log(
      `[Event Test] Testing individual event loading for device ${device_id}`
    );
    const events_response = await server_get_events_and_tags_for_device(
      device_id,
      1
    );
    if (events_response.status === EnumWebResponse.SUCCESS) {
      console.log(
        `[Event Test] Successfully loaded ${
          events_response.data?.length || 0
        } events for device ${device_id}`
      );
      return true;
    } else {
      console.error(
        `[Event Test] Failed to load events for device ${device_id}:`,
        events_response.msg
      );
      return false;
    }
  } catch (error) {
    console.error(
      `[Event Test] Failed to load events for device ${device_id}:`,
      error
    );
    return false;
  }
}

export async function server_create_tags(
  tags: ITag[]
): Promise<IWebResponseCompatible<ITag[]>> {
  const supabase = await newServerClient();
  // remove id key from tags
  const formatted_tags = tags.map((tag) => {
    const { id, ...rest } = tag;
    return {
      ...rest,
      observation_type: rest.observation_type as "manual" | "auto",
    };
  });

  const { data, error } = await supabase
    .from("tags")
    .insert(formatted_tags)
    .select("*");
  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  } else {
    return IWebResponse.success(data).to_compatible();
  }
}

export async function server_delete_tags_by_ids(
  tag_ids: number[]
): Promise<IWebResponseCompatible<boolean>> {
  const supabase = await newServerClient();
  const { error } = await supabase.from("tags").delete().in("id", tag_ids);
  if (error) {
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: false,
    };
  } else {
    return IWebResponse.success(true).to_compatible();
  }
}

export async function server_update_tags(
  tags: ITag[]
): Promise<IWebResponseCompatible<ITag[]>> {
  const supabase = await newServerClient();

  // Update each tag individually since we need to preserve the id
  const updatedTags: ITag[] = [];

  for (const tag of tags) {
    const { id, ...updateData } = tag;
    const { data, error } = await supabase
      .from("tags")
      .update({
        x: updateData.x,
        y: updateData.y,
        width: updateData.width,
        height: updateData.height,
        class_name: updateData.class_name,
        conf: updateData.conf,
        observation_type: updateData.observation_type as "manual" | "auto",
      })
      .eq("id", id)
      .select("*")
      .single();

    if (error) {
      return {
        status: EnumWebResponse.ERROR,
        msg: error.message,
        data: null,
      };
    }

    if (data) {
      updatedTags.push(data);
    }
  }

  return IWebResponse.success(updatedTags).to_compatible();
}

// export async function server_get_events_with_tags_by_herd(
//   herd_id: number
// ): Promise<IWebResponseCompatible<IEventWithTags[]>> {
//   const supabase = await newServerClient();
//   const { data, error } = await supabase
//     .from("events")
//     .select(
//       `
//       *,
//       tags: tags (*)
//       `
//     )
//     .eq("devices.herd_id", herd_id)
//     .order("timestamp_observation", { ascending: false });
//   if (error) {
//     return {
//       status: EnumWebResponse.ERROR,
//       msg: error.message,
//       data: [],
//     };
//   }
//   return IWebResponse.success(data).to_compatible();
// }

export async function server_get_more_events_with_tags_by_herd(
  herd_id: number,
  offset: number,
  page_count: number = 10
): Promise<IWebResponseCompatible<IEventWithTags[]>> {
  const from = offset * page_count;
  const to = from + page_count - 1;
  const supabase = await newServerClient();
  // make rpc call to get_events_with_tags_for_herd(herd_id, offset, limit)
  const { data, error } = await supabase.rpc("get_events_and_tags_for_herd", {
    herd_id_caller: herd_id,
    offset_caller: from,
    limit_caller: page_count,
  });
  if (error) {
    console.warn("Error fetching events with tags by herd:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  // iterate through data if tags contains null, remove it
  const filtered_data = (data || []).map((event: IEventWithTags) => {
    if (!event.tags) return event;
    event.tags = event.tags.filter((tag: ITag | null) => tag !== null);
    return event;
  });

  // Add signed URLs to events using the same client
  const eventsWithSignedUrls = await addSignedUrlsToEvents(
    filtered_data,
    supabase
  );

  return IWebResponse.success(eventsWithSignedUrls).to_compatible();
}

export async function server_get_events_and_tags_for_device(
  device_id: number,
  limit: number = 3
): Promise<IWebResponseCompatible<IEventWithTags[]>> {
  const supabase = await newServerClient();
  // make rpc call to get_events_with_tags_for_device(device_id, limit)
  const { data, error } = await supabase.rpc("get_events_and_tags_for_device", {
    device_id_caller: device_id,
    limit_caller: limit,
  });
  if (error) {
    console.warn(
      "Error fetching recent events with tags by device:",
      error.message
    );
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: [],
    };
  }

  // Add signed URLs to events using the same client
  const eventsWithSignedUrls = await addSignedUrlsToEvents(
    data || [],
    supabase
  );

  return IWebResponse.success(eventsWithSignedUrls).to_compatible();
}

export async function server_get_events_and_tags_for_devices_batch(
  device_ids: number[],
  limit: number = 1
): Promise<IWebResponseCompatible<{ [device_id: number]: IEventWithTags[] }>> {
  const supabase = await newServerClient();

  // Use single RPC call for all devices
  const { data, error } = await supabase.rpc(
    "get_events_and_tags_for_devices_batch",
    {
      device_ids: device_ids,
      limit_per_device: limit,
    }
  );

  if (error) {
    console.error(`[Events Batch] Database error:`, error.message);
    console.log(`[Events Batch] Falling back to individual calls...`);

    // Fallback to individual event loading
    const result: { [device_id: number]: IEventWithTags[] } = {};
    const promises = device_ids.map(async (device_id) => {
      try {
        const events_response = await server_get_events_and_tags_for_device(
          device_id,
          limit
        );
        if (
          events_response.status === EnumWebResponse.SUCCESS &&
          events_response.data
        ) {
          result[device_id] = events_response.data;
        } else {
          result[device_id] = [];
        }
      } catch (err) {
        console.warn(`[Events Batch] Failed for device ${device_id}:`, err);
        result[device_id] = [];
      }
    });

    await Promise.all(promises);
    return IWebResponse.success(result).to_compatible();
  }

  if (!data) {
    return IWebResponse.success({}).to_compatible();
  }

  // Group events by device_id
  const eventsByDevice: { [device_id: number]: any[] } = {};

  data.forEach((row: any) => {
    const device_id = row.device_id;
    if (!eventsByDevice[device_id]) {
      eventsByDevice[device_id] = [];
    }

    // Create event object from the event_and_tags_pretty_location structure
    const event = {
      id: row.id, // Changed from row.event_id to row.id
      inserted_at: row.inserted_at,
      message: row.message,
      media_url: row.media_url,
      file_path: row.file_path,
      latitude: row.latitude,
      longitude: row.longitude,
      altitude: row.altitude,
      heading: row.heading,
      media_type: row.media_type,
      device_id: device_id,
      timestamp_observation: row.timestamp_observation,
      is_public: row.is_public,
      earthranger_url: row.earthranger_url,
      tags: Array.isArray(row.tags) ? row.tags : [],
    };

    eventsByDevice[device_id].push(event);
  });

  // Add signed URLs to all events
  const result: { [device_id: number]: IEventWithTags[] } = {};

  for (const device_id in eventsByDevice) {
    const events = eventsByDevice[device_id];
    const eventsWithSignedUrls = await addSignedUrlsToEvents(events, supabase);
    result[parseInt(device_id)] = eventsWithSignedUrls;
  }

  return IWebResponse.success(result).to_compatible();
}

export async function get_event_and_tags_by_event_id(
  event_id: number
): Promise<IWebResponseCompatible<IEventWithTags>> {
  const supabase = await newServerClient();
  // use actual sql query to get event and tags instead of rpc
  const { data, error } = await supabase
    .from("events")
    .select(
      `
      *,
      tags: tags (*)
      `
    )
    .eq("id", event_id);
  if (error) {
    console.warn("Error fetching event with tags by event id:", error.message);
    return {
      status: EnumWebResponse.ERROR,
      msg: error.message,
      data: null,
    };
  }
  if (!data || data.length === 0) {
    return {
      status: EnumWebResponse.ERROR,
      msg: "Event not found",
      data: null,
    };
  }

  // Transform location to latitude/longitude with better error handling
  let latitude = null;
  let longitude = null;

  try {
    if (data[0].location) {
      if (typeof data[0].location === "object" && data[0].location !== null) {
        // Handle PostGIS Point format: { coordinates: [lon, lat] }
        if (
          "coordinates" in data[0].location &&
          Array.isArray(data[0].location.coordinates)
        ) {
          longitude = data[0].location.coordinates[0];
          latitude = data[0].location.coordinates[1];
        }
        // Handle alternative format: { x: lon, y: lat }
        else if ("x" in data[0].location && "y" in data[0].location) {
          longitude = data[0].location.x;
          latitude = data[0].location.y;
        }
      }
      // Handle string format: "Point(lon lat)"
      else if (typeof data[0].location === "string") {
        const match = data[0].location.match(/Point\(([^)]+)\)/);
        if (match) {
          const coords = match[1].split(" ").map(Number);
          if (coords.length === 2) {
            longitude = coords[0];
            latitude = coords[1];
          }
        }
      }
    }
  } catch (locationError) {
    console.warn("Error parsing location data:", locationError);
    // Continue with null coordinates
  }

  const transformedData: IEventWithTags = {
    id: data[0].id,
    inserted_at: data[0].inserted_at,
    message: data[0].message,
    media_url: data[0].media_url,
    latitude: latitude,
    longitude: longitude,
    altitude: data[0].altitude,
    heading: data[0].heading,
    media_type: data[0].media_type,
    device_id: data[0].device_id,
    timestamp_observation: data[0].timestamp_observation,
    is_public: data[0].is_public,
    tags: data[0].tags || [],
    earthranger_url: data[0].earthranger_url,
    file_path: data[0].file_path,
  };

  // Add signed URL to event using the same client
  const eventWithSignedUrl = await addSignedUrlToEvent(
    transformedData,
    supabase
  );

  return IWebResponse.success(eventWithSignedUrl).to_compatible();
}
