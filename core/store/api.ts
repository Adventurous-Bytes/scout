import { createApi, fakeBaseQuery } from "@reduxjs/toolkit/query/react";
import { SupabaseClient } from "@supabase/supabase-js";
import {
  ISession,
  IEvent,
  IArtifact,
  IArtifactWithMediaUrl,
  IEventWithTags,
  ISessionWithCoordinates,
  IEventAndTagsPrettyLocation,
} from "../types/db";
import { generateSignedUrlsBatch } from "../helpers/storage";

// Types for infinite query
export interface InfiniteQueryArgs {
  herdId?: number;
  deviceId?: number;
  limit?: number;
  cursor?: {
    timestamp: string;
    id: number;
  } | null;
}

export interface SessionsInfiniteResponse {
  sessions: ISessionWithCoordinates[];
  nextCursor: { timestamp: string; id: number } | null;
  hasMore: boolean;
}

export interface EventsInfiniteResponse {
  events: IEventAndTagsPrettyLocation[];
  nextCursor: { timestamp: string; id: number } | null;
  hasMore: boolean;
}

export interface ArtifactsInfiniteResponse {
  artifacts: IArtifactWithMediaUrl[];
  nextCursor: { timestamp: string; id: number } | null;
  hasMore: boolean;
}

// Create the API slice
export const scoutApi = createApi({
  reducerPath: "scoutApi",
  baseQuery: fakeBaseQuery(),
  tagTypes: ["Session", "Event", "Artifact"],
  endpoints: (builder) => ({
    // =====================================================
    // SESSIONS INFINITE QUERIES
    // =====================================================
    getSessionsInfiniteByHerd: builder.query<
      SessionsInfiniteResponse,
      InfiniteQueryArgs & { supabase: SupabaseClient }
    >({
      async queryFn({ herdId, limit = 20, cursor, supabase }) {
        try {
          if (!herdId) {
            return {
              error: { status: "CUSTOM_ERROR", error: "Herd ID is required" },
            };
          }

          const { data, error } = await supabase.rpc(
            "get_sessions_infinite_by_herd",
            {
              herd_id_caller: herdId,
              limit_caller: limit + 1, // Fetch one extra to determine if there are more
              cursor_timestamp: cursor?.timestamp || null,
              cursor_id: cursor?.id || null,
            },
          );

          if (error) {
            return {
              error: { status: "SUPABASE_ERROR", error: error.message },
            };
          }

          const sessions = (data as ISessionWithCoordinates[]) || [];
          const hasMore = sessions.length > limit;
          const resultSessions = hasMore ? sessions.slice(0, limit) : sessions;

          const nextCursor =
            hasMore && resultSessions.length > 0
              ? {
                  timestamp:
                    resultSessions[resultSessions.length - 1].timestamp_start ||
                    "",
                  id: resultSessions[resultSessions.length - 1].id || 0,
                }
              : null;

          return {
            data: {
              sessions: resultSessions,
              nextCursor,
              hasMore,
            },
          };
        } catch (err) {
          return { error: { status: "FETCH_ERROR", error: String(err) } };
        }
      },
      providesTags: (result) =>
        result
          ? [
              ...result.sessions.map(({ id }) => ({
                type: "Session" as const,
                id: id || "unknown",
              })),
              { type: "Session", id: "LIST" },
            ]
          : [{ type: "Session", id: "LIST" }],
    }),

    getSessionsInfiniteByDevice: builder.query<
      SessionsInfiniteResponse,
      InfiniteQueryArgs & { supabase: SupabaseClient }
    >({
      async queryFn({ deviceId, limit = 20, cursor, supabase }) {
        try {
          if (!deviceId) {
            return {
              error: { status: "CUSTOM_ERROR", error: "Device ID is required" },
            };
          }

          const { data, error } = await supabase.rpc(
            "get_sessions_infinite_by_device",
            {
              device_id_caller: deviceId,
              limit_caller: limit + 1,
              cursor_timestamp: cursor?.timestamp || null,
              cursor_id: cursor?.id || null,
            },
          );

          if (error) {
            return {
              error: { status: "SUPABASE_ERROR", error: error.message },
            };
          }

          const sessions = (data as ISessionWithCoordinates[]) || [];
          const hasMore = sessions.length > limit;
          const resultSessions = hasMore ? sessions.slice(0, limit) : sessions;

          const nextCursor =
            hasMore && resultSessions.length > 0
              ? {
                  timestamp:
                    resultSessions[resultSessions.length - 1].timestamp_start ||
                    "",
                  id: resultSessions[resultSessions.length - 1].id || 0,
                }
              : null;

          return {
            data: {
              sessions: resultSessions,
              nextCursor,
              hasMore,
            },
          };
        } catch (err) {
          return { error: { status: "FETCH_ERROR", error: String(err) } };
        }
      },
      providesTags: (result) =>
        result
          ? [
              ...result.sessions.map(({ id }) => ({
                type: "Session" as const,
                id: id || "unknown",
              })),
              { type: "Session", id: "LIST" },
            ]
          : [{ type: "Session", id: "LIST" }],
    }),

    // =====================================================
    // EVENTS INFINITE QUERIES
    // =====================================================
    getEventsInfiniteByHerd: builder.query<
      EventsInfiniteResponse,
      InfiniteQueryArgs & { supabase: SupabaseClient }
    >({
      async queryFn({ herdId, limit = 20, cursor, supabase }) {
        try {
          if (!herdId) {
            return {
              error: { status: "CUSTOM_ERROR", error: "Herd ID is required" },
            };
          }

          const { data, error } = await supabase.rpc(
            "get_events_infinite_by_herd",
            {
              herd_id_caller: herdId,
              limit_caller: limit + 1,
              cursor_timestamp: cursor?.timestamp || null,
              cursor_id: cursor?.id || null,
            },
          );

          if (error) {
            return {
              error: { status: "SUPABASE_ERROR", error: error.message },
            };
          }

          const events = (data as IEventAndTagsPrettyLocation[]) || [];
          const hasMore = events.length > limit;
          const resultEvents = hasMore ? events.slice(0, limit) : events;

          const nextCursor =
            hasMore && resultEvents.length > 0
              ? {
                  timestamp:
                    resultEvents[resultEvents.length - 1]
                      .timestamp_observation || "",
                  id: resultEvents[resultEvents.length - 1].id || 0,
                }
              : null;

          return {
            data: {
              events: resultEvents,
              nextCursor,
              hasMore,
            },
          };
        } catch (err) {
          return { error: { status: "FETCH_ERROR", error: String(err) } };
        }
      },
      providesTags: (result) =>
        result
          ? [
              ...result.events.map(({ id }) => ({
                type: "Event" as const,
                id: id || "unknown",
              })),
              { type: "Event", id: "LIST" },
            ]
          : [{ type: "Event", id: "LIST" }],
    }),

    getEventsInfiniteByDevice: builder.query<
      EventsInfiniteResponse,
      InfiniteQueryArgs & { supabase: SupabaseClient }
    >({
      async queryFn({ deviceId, limit = 20, cursor, supabase }) {
        try {
          if (!deviceId) {
            return {
              error: { status: "CUSTOM_ERROR", error: "Device ID is required" },
            };
          }

          const { data, error } = await supabase.rpc(
            "get_events_infinite_by_device",
            {
              device_id_caller: deviceId,
              limit_caller: limit + 1,
              cursor_timestamp: cursor?.timestamp || null,
              cursor_id: cursor?.id || null,
            },
          );

          if (error) {
            return {
              error: { status: "SUPABASE_ERROR", error: error.message },
            };
          }

          const events = (data as IEventAndTagsPrettyLocation[]) || [];
          const hasMore = events.length > limit;
          const resultEvents = hasMore ? events.slice(0, limit) : events;

          const nextCursor =
            hasMore && resultEvents.length > 0
              ? {
                  timestamp:
                    resultEvents[resultEvents.length - 1]
                      .timestamp_observation || "",
                  id: resultEvents[resultEvents.length - 1].id || 0,
                }
              : null;

          return {
            data: {
              events: resultEvents,
              nextCursor,
              hasMore,
            },
          };
        } catch (err) {
          return { error: { status: "FETCH_ERROR", error: String(err) } };
        }
      },
      providesTags: (result) =>
        result
          ? [
              ...result.events.map(({ id }) => ({
                type: "Event" as const,
                id: id || "unknown",
              })),
              { type: "Event", id: "LIST" },
            ]
          : [{ type: "Event", id: "LIST" }],
    }),

    // =====================================================
    // ARTIFACTS INFINITE QUERIES
    // =====================================================
    getArtifactsInfiniteByHerd: builder.query<
      ArtifactsInfiniteResponse,
      InfiniteQueryArgs & { supabase: SupabaseClient }
    >({
      async queryFn({ herdId, limit = 20, cursor, supabase }) {
        try {
          if (!herdId) {
            return {
              error: { status: "CUSTOM_ERROR", error: "Herd ID is required" },
            };
          }

          const { data, error } = await supabase.rpc(
            "get_artifacts_infinite_by_herd",
            {
              herd_id_caller: herdId,
              limit_caller: limit + 1,
              cursor_timestamp: cursor?.timestamp || null,
              cursor_id: cursor?.id || null,
            },
          );

          if (error) {
            return {
              error: { status: "SUPABASE_ERROR", error: error.message },
            };
          }

          const artifacts = (data as IArtifact[]) || [];
          const hasMore = artifacts.length > limit;
          const resultArtifacts = hasMore
            ? artifacts.slice(0, limit)
            : artifacts;

          // Generate signed URLs for artifacts
          const uniqueFilePaths = Array.from(
            new Set(
              resultArtifacts
                .map((artifact) => artifact.file_path)
                .filter(
                  (path): path is string => path !== null && path !== undefined,
                ),
            ),
          );

          let urlMap = new Map<string, string>();
          if (uniqueFilePaths.length > 0) {
            try {
              const urlResults = await generateSignedUrlsBatch(uniqueFilePaths);
              urlResults.forEach((url, index) => {
                if (url) {
                  urlMap.set(uniqueFilePaths[index], url);
                }
              });
            } catch (urlError) {
              console.warn(
                "Failed to generate signed URLs for artifacts:",
                urlError,
              );
            }
          }

          const artifactsWithUrls: IArtifactWithMediaUrl[] =
            resultArtifacts.map((artifact) => ({
              ...artifact,
              media_url: artifact.file_path
                ? urlMap.get(artifact.file_path) || null
                : null,
            }));

          const nextCursor =
            hasMore && resultArtifacts.length > 0
              ? {
                  timestamp:
                    resultArtifacts[resultArtifacts.length - 1].created_at,
                  id: resultArtifacts[resultArtifacts.length - 1].id,
                }
              : null;

          return {
            data: {
              artifacts: artifactsWithUrls,
              nextCursor,
              hasMore,
            },
          };
        } catch (err) {
          return { error: { status: "FETCH_ERROR", error: String(err) } };
        }
      },
      providesTags: (result) =>
        result
          ? [
              ...result.artifacts.map(({ id }) => ({
                type: "Artifact" as const,
                id,
              })),
              { type: "Artifact", id: "LIST" },
            ]
          : [{ type: "Artifact", id: "LIST" }],
    }),

    getArtifactsInfiniteByDevice: builder.query<
      ArtifactsInfiniteResponse,
      InfiniteQueryArgs & { supabase: SupabaseClient }
    >({
      async queryFn({ deviceId, limit = 20, cursor, supabase }) {
        try {
          if (!deviceId) {
            return {
              error: { status: "CUSTOM_ERROR", error: "Device ID is required" },
            };
          }

          const { data, error } = await supabase.rpc(
            "get_artifacts_infinite_by_device",
            {
              device_id_caller: deviceId,
              limit_caller: limit + 1,
              cursor_timestamp: cursor?.timestamp || null,
              cursor_id: cursor?.id || null,
            },
          );

          if (error) {
            return {
              error: { status: "SUPABASE_ERROR", error: error.message },
            };
          }

          const artifacts = (data as IArtifact[]) || [];
          const hasMore = artifacts.length > limit;
          const resultArtifacts = hasMore
            ? artifacts.slice(0, limit)
            : artifacts;

          // Generate signed URLs for artifacts
          const uniqueFilePaths = Array.from(
            new Set(
              resultArtifacts
                .map((artifact) => artifact.file_path)
                .filter(
                  (path): path is string => path !== null && path !== undefined,
                ),
            ),
          );

          let urlMap = new Map<string, string>();
          if (uniqueFilePaths.length > 0) {
            try {
              const urlResults = await generateSignedUrlsBatch(uniqueFilePaths);
              urlResults.forEach((url, index) => {
                if (url) {
                  urlMap.set(uniqueFilePaths[index], url);
                }
              });
            } catch (urlError) {
              console.warn(
                "Failed to generate signed URLs for artifacts:",
                urlError,
              );
            }
          }

          const artifactsWithUrls: IArtifactWithMediaUrl[] =
            resultArtifacts.map((artifact) => ({
              ...artifact,
              media_url: artifact.file_path
                ? urlMap.get(artifact.file_path) || null
                : null,
            }));

          const nextCursor =
            hasMore && resultArtifacts.length > 0
              ? {
                  timestamp:
                    resultArtifacts[resultArtifacts.length - 1].created_at,
                  id: resultArtifacts[resultArtifacts.length - 1].id,
                }
              : null;

          return {
            data: {
              artifacts: artifactsWithUrls,
              nextCursor,
              hasMore,
            },
          };
        } catch (err) {
          return { error: { status: "FETCH_ERROR", error: String(err) } };
        }
      },
      providesTags: (result) =>
        result
          ? [
              ...result.artifacts.map(({ id }) => ({
                type: "Artifact" as const,
                id,
              })),
              { type: "Artifact", id: "LIST" },
            ]
          : [{ type: "Artifact", id: "LIST" }],
    }),
  }),
});

// Export hooks for usage in functional components
export const {
  useGetSessionsInfiniteByHerdQuery,
  useGetSessionsInfiniteByDeviceQuery,
  useGetEventsInfiniteByHerdQuery,
  useGetEventsInfiniteByDeviceQuery,
  useGetArtifactsInfiniteByHerdQuery,
  useGetArtifactsInfiniteByDeviceQuery,
} = scoutApi;
