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
