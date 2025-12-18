import { useState, useCallback, useMemo, useEffect } from "react";
import { SupabaseClient } from "@supabase/supabase-js";
import {
  useGetSessionsInfiniteByHerdQuery,
  useGetSessionsInfiniteByDeviceQuery,
  useGetEventsInfiniteByHerdQuery,
  useGetEventsInfiniteByDeviceQuery,
  useGetArtifactsInfiniteByHerdQuery,
  useGetArtifactsInfiniteByDeviceQuery,
  InfiniteQueryArgs,
} from "../store/api";
import {
  ISession,
  IEventWithTags,
  IArtifactWithMediaUrl,
  ISessionWithCoordinates,
  IEventAndTagsPrettyLocation,
} from "../types/db";

interface UseInfiniteScrollOptions {
  limit?: number;
  enabled?: boolean;
  supabase: SupabaseClient;
}

interface InfiniteScrollData<T> {
  items: T[];
  isLoading: boolean;
  isLoadingMore: boolean;
  hasMore: boolean;
  loadMore: () => void;
  refetch: () => void;
  error: any;
}

// =====================================================
// SESSIONS INFINITE SCROLL HOOKS
// =====================================================

export const useInfiniteSessionsByHerd = (
  herdId: number,
  options: UseInfiniteScrollOptions,
): InfiniteScrollData<ISessionWithCoordinates> => {
  const [pages, setPages] = useState<
    Array<{
      cursor: { timestamp: string; id: number } | null;
      data: ISessionWithCoordinates[];
    }>
  >([]);
  const [currentCursor, setCurrentCursor] = useState<{
    timestamp: string;
    id: number;
  } | null>(null);

  const currentQuery = useGetSessionsInfiniteByHerdQuery(
    {
      herdId,
      limit: options.limit || 20,
      cursor: currentCursor,
      supabase: options.supabase,
    },
    {
      skip: !options.enabled,
    },
  );

  // Update pages when new data arrives
  useEffect(() => {
    if (currentQuery.data && !currentQuery.isLoading) {
      setPages((prev) => {
        const existingPage = prev.find(
          (p) =>
            (p.cursor === null && currentCursor === null) ||
            (p.cursor &&
              currentCursor &&
              p.cursor.id === currentCursor.id &&
              p.cursor.timestamp === currentCursor.timestamp),
        );

        if (!existingPage) {
          return [
            ...prev,
            { cursor: currentCursor, data: currentQuery.data!.sessions },
          ];
        }
        return prev;
      });
    }
  }, [currentQuery.data, currentQuery.isLoading, currentCursor]);

  const loadMore = useCallback(() => {
    if (
      currentQuery.data?.hasMore &&
      currentQuery.data.nextCursor &&
      !currentQuery.isLoading
    ) {
      setCurrentCursor(currentQuery.data.nextCursor);
    }
  }, [currentQuery.data, currentQuery.isLoading]);

  const refetch = useCallback(() => {
    setPages([]);
    setCurrentCursor(null);
    currentQuery.refetch();
  }, [currentQuery]);

  // Flatten all pages into single array
  const allItems = useMemo(() => {
    return pages.flatMap((page) => page.data);
  }, [pages]);

  return {
    items: allItems,
    isLoading: currentQuery.isLoading && pages.length === 0,
    isLoadingMore: currentQuery.isLoading && pages.length > 0,
    hasMore: currentQuery.data?.hasMore || false,
    loadMore,
    refetch,
    error: currentQuery.error,
  };
};

export const useInfiniteSessionsByDevice = (
  deviceId: number,
  options: UseInfiniteScrollOptions,
): InfiniteScrollData<ISessionWithCoordinates> => {
  const [pages, setPages] = useState<
    Array<{
      cursor: { timestamp: string; id: number } | null;
      data: ISessionWithCoordinates[];
    }>
  >([]);
  const [currentCursor, setCurrentCursor] = useState<{
    timestamp: string;
    id: number;
  } | null>(null);

  const currentQuery = useGetSessionsInfiniteByDeviceQuery(
    {
      deviceId,
      limit: options.limit || 20,
      cursor: currentCursor,
      supabase: options.supabase,
    },
    {
      skip: !options.enabled,
    },
  );

  useEffect(() => {
    if (currentQuery.data && !currentQuery.isLoading) {
      setPages((prev) => {
        const existingPage = prev.find(
          (p) =>
            (p.cursor === null && currentCursor === null) ||
            (p.cursor &&
              currentCursor &&
              p.cursor.id === currentCursor.id &&
              p.cursor.timestamp === currentCursor.timestamp),
        );

        if (!existingPage) {
          return [
            ...prev,
            { cursor: currentCursor, data: currentQuery.data!.sessions },
          ];
        }
        return prev;
      });
    }
  }, [currentQuery.data, currentQuery.isLoading, currentCursor]);

  const loadMore = useCallback(() => {
    if (
      currentQuery.data?.hasMore &&
      currentQuery.data.nextCursor &&
      !currentQuery.isLoading
    ) {
      setCurrentCursor(currentQuery.data.nextCursor);
    }
  }, [currentQuery.data, currentQuery.isLoading]);

  const refetch = useCallback(() => {
    setPages([]);
    setCurrentCursor(null);
    currentQuery.refetch();
  }, [currentQuery]);

  const allItems = useMemo(() => {
    return pages.flatMap((page) => page.data);
  }, [pages]);

  return {
    items: allItems,
    isLoading: currentQuery.isLoading && pages.length === 0,
    isLoadingMore: currentQuery.isLoading && pages.length > 0,
    hasMore: currentQuery.data?.hasMore || false,
    loadMore,
    refetch,
    error: currentQuery.error,
  };
};

// =====================================================
// EVENTS INFINITE SCROLL HOOKS
// =====================================================

export const useInfiniteEventsByHerd = (
  herdId: number,
  options: UseInfiniteScrollOptions,
): InfiniteScrollData<IEventAndTagsPrettyLocation> => {
  const [pages, setPages] = useState<
    Array<{
      cursor: { timestamp: string; id: number } | null;
      data: IEventAndTagsPrettyLocation[];
    }>
  >([]);
  const [currentCursor, setCurrentCursor] = useState<{
    timestamp: string;
    id: number;
  } | null>(null);

  const currentQuery = useGetEventsInfiniteByHerdQuery(
    {
      herdId,
      limit: options.limit || 20,
      cursor: currentCursor,
      supabase: options.supabase,
    },
    {
      skip: !options.enabled,
    },
  );

  useEffect(() => {
    if (currentQuery.data && !currentQuery.isLoading) {
      setPages((prev) => {
        const existingPage = prev.find(
          (p) =>
            (p.cursor === null && currentCursor === null) ||
            (p.cursor &&
              currentCursor &&
              p.cursor.id === currentCursor.id &&
              p.cursor.timestamp === currentCursor.timestamp),
        );

        if (!existingPage) {
          return [
            ...prev,
            { cursor: currentCursor, data: currentQuery.data!.events },
          ];
        }
        return prev;
      });
    }
  }, [currentQuery.data, currentQuery.isLoading, currentCursor]);

  const loadMore = useCallback(() => {
    if (
      currentQuery.data?.hasMore &&
      currentQuery.data.nextCursor &&
      !currentQuery.isLoading
    ) {
      setCurrentCursor(currentQuery.data.nextCursor);
    }
  }, [currentQuery.data, currentQuery.isLoading]);

  const refetch = useCallback(() => {
    setPages([]);
    setCurrentCursor(null);
    currentQuery.refetch();
  }, [currentQuery]);

  const allItems = useMemo(() => {
    return pages.flatMap((page) => page.data);
  }, [pages]);

  return {
    items: allItems,
    isLoading: currentQuery.isLoading && pages.length === 0,
    isLoadingMore: currentQuery.isLoading && pages.length > 0,
    hasMore: currentQuery.data?.hasMore || false,
    loadMore,
    refetch,
    error: currentQuery.error,
  };
};

export const useInfiniteEventsByDevice = (
  deviceId: number,
  options: UseInfiniteScrollOptions,
): InfiniteScrollData<IEventAndTagsPrettyLocation> => {
  const [pages, setPages] = useState<
    Array<{
      cursor: { timestamp: string; id: number } | null;
      data: IEventAndTagsPrettyLocation[];
    }>
  >([]);
  const [currentCursor, setCurrentCursor] = useState<{
    timestamp: string;
    id: number;
  } | null>(null);

  const currentQuery = useGetEventsInfiniteByDeviceQuery(
    {
      deviceId,
      limit: options.limit || 20,
      cursor: currentCursor,
      supabase: options.supabase,
    },
    {
      skip: !options.enabled,
    },
  );

  useEffect(() => {
    if (currentQuery.data && !currentQuery.isLoading) {
      setPages((prev) => {
        const existingPage = prev.find(
          (p) =>
            (p.cursor === null && currentCursor === null) ||
            (p.cursor &&
              currentCursor &&
              p.cursor.id === currentCursor.id &&
              p.cursor.timestamp === currentCursor.timestamp),
        );

        if (!existingPage) {
          return [
            ...prev,
            { cursor: currentCursor, data: currentQuery.data!.events },
          ];
        }
        return prev;
      });
    }
  }, [currentQuery.data, currentQuery.isLoading, currentCursor]);

  const loadMore = useCallback(() => {
    if (
      currentQuery.data?.hasMore &&
      currentQuery.data.nextCursor &&
      !currentQuery.isLoading
    ) {
      setCurrentCursor(currentQuery.data.nextCursor);
    }
  }, [currentQuery.data, currentQuery.isLoading]);

  const refetch = useCallback(() => {
    setPages([]);
    setCurrentCursor(null);
    currentQuery.refetch();
  }, [currentQuery]);

  const allItems = useMemo(() => {
    return pages.flatMap((page) => page.data);
  }, [pages]);

  return {
    items: allItems,
    isLoading: currentQuery.isLoading && pages.length === 0,
    isLoadingMore: currentQuery.isLoading && pages.length > 0,
    hasMore: currentQuery.data?.hasMore || false,
    loadMore,
    refetch,
    error: currentQuery.error,
  };
};

// =====================================================
// ARTIFACTS INFINITE SCROLL HOOKS
// =====================================================

export const useInfiniteArtifactsByHerd = (
  herdId: number,
  options: UseInfiniteScrollOptions,
): InfiniteScrollData<IArtifactWithMediaUrl> => {
  const [pages, setPages] = useState<
    Array<{
      cursor: { timestamp: string; id: number } | null;
      data: IArtifactWithMediaUrl[];
    }>
  >([]);
  const [currentCursor, setCurrentCursor] = useState<{
    timestamp: string;
    id: number;
  } | null>(null);

  const currentQuery = useGetArtifactsInfiniteByHerdQuery(
    {
      herdId,
      limit: options.limit || 20,
      cursor: currentCursor,
      supabase: options.supabase,
    },
    {
      skip: !options.enabled,
    },
  );

  useEffect(() => {
    if (currentQuery.data && !currentQuery.isLoading) {
      setPages((prev) => {
        const existingPage = prev.find(
          (p) =>
            (p.cursor === null && currentCursor === null) ||
            (p.cursor &&
              currentCursor &&
              p.cursor.id === currentCursor.id &&
              p.cursor.timestamp === currentCursor.timestamp),
        );

        if (!existingPage) {
          return [
            ...prev,
            { cursor: currentCursor, data: currentQuery.data!.artifacts },
          ];
        }
        return prev;
      });
    }
  }, [currentQuery.data, currentQuery.isLoading, currentCursor]);

  const loadMore = useCallback(() => {
    if (
      currentQuery.data?.hasMore &&
      currentQuery.data.nextCursor &&
      !currentQuery.isLoading
    ) {
      setCurrentCursor(currentQuery.data.nextCursor);
    }
  }, [currentQuery.data, currentQuery.isLoading]);

  const refetch = useCallback(() => {
    setPages([]);
    setCurrentCursor(null);
    currentQuery.refetch();
  }, [currentQuery]);

  const allItems = useMemo(() => {
    return pages.flatMap((page) => page.data);
  }, [pages]);

  return {
    items: allItems,
    isLoading: currentQuery.isLoading && pages.length === 0,
    isLoadingMore: currentQuery.isLoading && pages.length > 0,
    hasMore: currentQuery.data?.hasMore || false,
    loadMore,
    refetch,
    error: currentQuery.error,
  };
};

export const useInfiniteArtifactsByDevice = (
  deviceId: number,
  options: UseInfiniteScrollOptions,
): InfiniteScrollData<IArtifactWithMediaUrl> => {
  const [pages, setPages] = useState<
    Array<{
      cursor: { timestamp: string; id: number } | null;
      data: IArtifactWithMediaUrl[];
    }>
  >([]);
  const [currentCursor, setCurrentCursor] = useState<{
    timestamp: string;
    id: number;
  } | null>(null);

  const currentQuery = useGetArtifactsInfiniteByDeviceQuery(
    {
      deviceId,
      limit: options.limit || 20,
      cursor: currentCursor,
      supabase: options.supabase,
    },
    {
      skip: !options.enabled,
    },
  );

  useEffect(() => {
    if (currentQuery.data && !currentQuery.isLoading) {
      setPages((prev) => {
        const existingPage = prev.find(
          (p) =>
            (p.cursor === null && currentCursor === null) ||
            (p.cursor &&
              currentCursor &&
              p.cursor.id === currentCursor.id &&
              p.cursor.timestamp === currentCursor.timestamp),
        );

        if (!existingPage) {
          return [
            ...prev,
            { cursor: currentCursor, data: currentQuery.data!.artifacts },
          ];
        }
        return prev;
      });
    }
  }, [currentQuery.data, currentQuery.isLoading, currentCursor]);

  const loadMore = useCallback(() => {
    if (
      currentQuery.data?.hasMore &&
      currentQuery.data.nextCursor &&
      !currentQuery.isLoading
    ) {
      setCurrentCursor(currentQuery.data.nextCursor);
    }
  }, [currentQuery.data, currentQuery.isLoading]);

  const refetch = useCallback(() => {
    setPages([]);
    setCurrentCursor(null);
    currentQuery.refetch();
  }, [currentQuery]);

  const allItems = useMemo(() => {
    return pages.flatMap((page) => page.data);
  }, [pages]);

  return {
    items: allItems,
    isLoading: currentQuery.isLoading && pages.length === 0,
    isLoadingMore: currentQuery.isLoading && pages.length > 0,
    hasMore: currentQuery.data?.hasMore || false,
    loadMore,
    refetch,
    error: currentQuery.error,
  };
};

// =====================================================
// INTERSECTION OBSERVER HOOK FOR AUTO-LOADING
// =====================================================

export const useIntersectionObserver = (
  callback: () => void,
  options: IntersectionObserverInit = {},
) => {
  const [element, setElement] = useState<Element | null>(null);

  useEffect(() => {
    if (!element) return;

    const observer = new IntersectionObserver(
      (entries) => {
        const first = entries[0];
        if (first.isIntersecting) {
          callback();
        }
      },
      { threshold: 0.1, ...options },
    );

    observer.observe(element);

    return () => {
      if (element) {
        observer.unobserve(element);
      }
    };
  }, [element, callback, options]);

  return setElement;
};
