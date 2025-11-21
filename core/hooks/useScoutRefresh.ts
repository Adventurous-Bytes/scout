import { useEffect, useCallback, useRef, useMemo } from "react";
import { useAppDispatch } from "../store/hooks";
import { useStore } from "react-redux";
import { RootState } from "../store/scout";
import {
  EnumScoutStateStatus,
  setHerdModules,
  setStatus,
  setHerdModulesLoadingState,
  setHerdModulesLoadedInMs,
  setHerdModulesApiServerProcessingDuration,
  setHerdModulesApiTotalRequestDuration,
  setUserApiDuration,
  setDataProcessingDuration,
  setCacheLoadDuration,
  setUser,
  setDataSource,
  setDataSourceInfo,
} from "../store/scout";
import { EnumHerdModulesLoadingState } from "../types/herd_module";
import { server_load_herd_modules } from "../helpers/herds";
import { scoutCache } from "../helpers/cache";
import { EnumDataSource } from "../types/data_source";
import { createBrowserClient } from "@supabase/ssr";
import { Database } from "../types/supabase";

export interface UseScoutRefreshOptions {
  autoRefresh?: boolean;
  onRefreshComplete?: () => void;
  cacheFirst?: boolean; // New option to enable cache-first loading
  cacheTtlMs?: number; // Cache TTL in milliseconds (default: 24 hours)
}

/**
 * Hook for refreshing scout data with detailed timing measurements and cache-first loading
 *
 * @param options - Configuration options for the refresh behavior
 * @param options.autoRefresh - Whether to automatically refresh on mount (default: true)
 * @param options.onRefreshComplete - Callback function called when refresh completes
 * @param options.cacheFirst - Whether to load from cache first, then refresh (default: true)
 * @param options.cacheTtlMs - Cache time-to-live in milliseconds (default: 24 hours)
 *
 * @returns Object containing:
 * - handleRefresh: Function to manually trigger a refresh
 * - clearCache: Function to clear the cache
 *
 * @example
 * ```tsx
 * const { handleRefresh, clearCache } = useScoutRefresh({
 *   cacheFirst: true,
 *   cacheTtlMs: 10 * 60 * 1000 // 10 minutes
 * });
 *
 * // Timing stats are available in Redux store via selectors
 * ```
 */
export function useScoutRefresh(options: UseScoutRefreshOptions = {}) {
  const {
    autoRefresh = true,
    onRefreshComplete,
    cacheFirst = true,
    cacheTtlMs = 24 * 60 * 60 * 1000, // 24 hours default (1 day)
  } = options;
  const dispatch = useAppDispatch();
  const store = useStore<RootState>();
  const refreshInProgressRef = useRef(false);

  // Create Supabase client directly to avoid circular dependency
  const supabase = useMemo(() => {
    return createBrowserClient<Database>(
      process.env.NEXT_PUBLIC_SUPABASE_URL || "",
      process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || "",
    );
  }, []);

  // Refs to store timing measurements
  const timingRefs = useRef({
    startTime: 0,
    herdModulesDuration: 0,
    userApiDuration: 0,
    dataProcessingDuration: 0,
    cacheLoadDuration: 0,
    cacheSaveDuration: 0,
  });

  // Helper function for deep comparison of objects
  const deepEqual = useCallback(
    (obj1: any, obj2: any, visited = new WeakMap()): boolean => {
      if (obj1 === obj2) return true;

      if (obj1 == null || obj2 == null) return obj1 === obj2;

      if (typeof obj1 !== typeof obj2) return false;

      if (typeof obj1 !== "object") return obj1 === obj2;

      // Handle circular references
      if (visited.has(obj1)) {
        return visited.get(obj1) === obj2;
      }
      visited.set(obj1, obj2);

      // Handle Date objects
      if (obj1 instanceof Date && obj2 instanceof Date) {
        return obj1.getTime() === obj2.getTime();
      }

      if (Array.isArray(obj1) !== Array.isArray(obj2)) return false;

      if (Array.isArray(obj1)) {
        if (obj1.length !== obj2.length) return false;
        for (let i = 0; i < obj1.length; i++) {
          if (!deepEqual(obj1[i], obj2[i], visited)) return false;
        }
        return true;
      }

      const keys1 = Object.keys(obj1);
      const keys2 = Object.keys(obj2);

      if (keys1.length !== keys2.length) return false;

      for (const key of keys1) {
        if (!keys2.includes(key)) return false;
        if (!deepEqual(obj1[key], obj2[key], visited)) return false;
      }

      return true;
    },
    [],
  );

  // Helper function to sort herd modules consistently by ID
  const sortHerdModulesById = useCallback((herdModules: any[]) => {
    if (!Array.isArray(herdModules)) return herdModules;

    return [...herdModules].sort((a, b) => {
      const aId = a?.herd?.id || 0;
      const bId = b?.herd?.id || 0;
      return aId - bId;
    });
  }, []);

  // Helper function to normalize herd modules for comparison (excludes timestamp metadata)
  const normalizeHerdModulesForComparison = useCallback(
    (herdModules: any[]) => {
      if (!Array.isArray(herdModules)) return herdModules;

      return herdModules.map((hm) => {
        if (!hm || typeof hm !== "object") return hm;

        // Create a copy without timestamp metadata that doesn't represent business data changes
        const { timestamp_last_refreshed, ...businessData } = hm;
        return businessData;
      });
    },
    [],
  );

  // Helper function to find what specifically changed for debugging
  const findBusinessDataChanges = useCallback(
    (newData: any[], currentData: any[]) => {
      if (!Array.isArray(newData) || !Array.isArray(currentData)) {
        return `Array type mismatch: new=${Array.isArray(newData)}, current=${Array.isArray(currentData)}`;
      }

      if (newData.length !== currentData.length) {
        return `Array length: ${currentData.length} → ${newData.length}`;
      }

      // Sort and normalize both for consistent comparison
      const sortedNew = normalizeHerdModulesForComparison(
        sortHerdModulesById(newData),
      );
      const sortedCurrent = normalizeHerdModulesForComparison(
        sortHerdModulesById(currentData),
      );

      const changes: string[] = [];

      for (let i = 0; i < sortedNew.length; i++) {
        const newHerd = sortedNew[i];
        const currentHerd = sortedCurrent[i];

        if (!newHerd || !currentHerd) continue;

        const herdName =
          newHerd.herd?.name || newHerd.name || `herd-${newHerd.herd?.id || i}`;

        // Check key business fields
        const businessFields = [
          "total_events",
          "total_events_with_filters",
          "events_page_index",
        ];

        businessFields.forEach((field) => {
          if (newHerd[field] !== currentHerd[field]) {
            changes.push(
              `${herdName}.${field}: ${currentHerd[field]} → ${newHerd[field]}`,
            );
          }
        });

        // Check array lengths
        const arrayFields = [
          "devices",
          "events",
          "plans",
          "zones",
          "sessions",
          "layers",
          "providers",
        ];
        arrayFields.forEach((field) => {
          const newArray = newHerd[field];
          const currentArray = currentHerd[field];
          if (Array.isArray(newArray) && Array.isArray(currentArray)) {
            if (newArray.length !== currentArray.length) {
              changes.push(
                `${herdName}.${field}[]: ${currentArray.length} → ${newArray.length}`,
              );
            }
          }
        });
      }

      return changes.length > 0
        ? changes.join(", ")
        : "No specific changes identified";
    },
    [normalizeHerdModulesForComparison, sortHerdModulesById],
  );

  // Helper function to conditionally dispatch only if business data has changed
  const conditionalDispatch = useCallback(
    (
      newData: any,
      currentData: any,
      actionCreator: (data: any) => any,
      dataType: string,
      skipTimestampOnlyUpdates: boolean = true,
    ) => {
      // For herd modules, sort both datasets by ID before comparison
      let dataToCompare = newData;
      let currentToCompare = currentData;

      if (dataType.includes("Herd modules")) {
        dataToCompare = sortHerdModulesById(newData);
        currentToCompare = sortHerdModulesById(currentData);

        // If we want to skip timestamp-only updates, normalize the data for comparison
        if (skipTimestampOnlyUpdates) {
          dataToCompare = normalizeHerdModulesForComparison(dataToCompare);
          currentToCompare =
            normalizeHerdModulesForComparison(currentToCompare);
        }
      }

      if (!deepEqual(dataToCompare, currentToCompare)) {
        console.log(
          `[useScoutRefresh] ${dataType} business data changed, updating store`,
        );

        // Add debugging for unexpected business changes
        if (skipTimestampOnlyUpdates && dataType.includes("Herd modules")) {
          const changes = findBusinessDataChanges(
            dataToCompare,
            currentToCompare,
          );
          console.log(`[useScoutRefresh] ${dataType} changes: ${changes}`);
        }

        dispatch(actionCreator(newData)); // Always dispatch original unsorted data
        return true;
      } else {
        console.log(
          `[useScoutRefresh] ${dataType} business data unchanged, skipping store update`,
        );
        return false;
      }
    },
    [
      dispatch,
      deepEqual,
      sortHerdModulesById,
      normalizeHerdModulesForComparison,
      findBusinessDataChanges,
    ],
  );

  // Helper function to handle IndexedDB errors - memoized for stability
  const handleIndexedDbError = useCallback(
    async (
      error: unknown,
      operation: string,
      retryFn?: () => Promise<void>,
    ) => {
      if (
        error instanceof Error &&
        (error.message.includes("object store") ||
          error.message.includes("NotFoundError"))
      ) {
        console.log(
          `[useScoutRefresh] Attempting database reset due to ${operation} error...`,
        );
        try {
          await scoutCache.resetDatabase();
          console.log("[useScoutRefresh] Database reset successful");
          if (retryFn) {
            await retryFn();
            console.log(
              `[useScoutRefresh] ${operation} successful after database reset`,
            );
          }
        } catch (resetError) {
          console.error(
            `[useScoutRefresh] Database reset and retry failed:`,
            resetError,
          );
        }
      }
    },
    [],
  );

  const handleRefresh = useCallback(async () => {
    // Prevent concurrent refresh calls
    if (refreshInProgressRef.current) {
      console.warn("[useScoutRefresh] Refresh already in progress, skipping");
      return;
    }

    refreshInProgressRef.current = true;
    const startTime = Date.now();
    timingRefs.current.startTime = startTime;

    try {
      dispatch(setStatus(EnumScoutStateStatus.LOADING));
      dispatch(setHerdModulesLoadingState(EnumHerdModulesLoadingState.LOADING));

      let cachedHerdModules: any[] | null = null;
      let cacheLoadDuration = 0;

      // Step 1: Load from cache first if enabled
      if (cacheFirst) {
        const cacheStartTime = Date.now();
        try {
          const cacheResult = await scoutCache.getHerdModules();
          cacheLoadDuration = Date.now() - cacheStartTime;
          timingRefs.current.cacheLoadDuration = cacheLoadDuration;
          dispatch(setCacheLoadDuration(cacheLoadDuration));

          if (cacheResult.data && cacheResult.data.length > 0) {
            cachedHerdModules = cacheResult.data;
            console.log(
              `[useScoutRefresh] Loaded ${
                cachedHerdModules.length
              } herd modules from cache in ${cacheLoadDuration}ms (age: ${Math.round(
                cacheResult.age / 1000,
              )}s, stale: ${cacheResult.isStale})`,
            );

            // Set data source to CACHE initially
            dispatch(setDataSource(EnumDataSource.CACHE));
            dispatch(
              setDataSourceInfo({
                source: EnumDataSource.CACHE,
                timestamp: Date.now(),
                cacheAge: cacheResult.age,
                isStale: cacheResult.isStale,
              }),
            );

            // Conditionally update the store with cached data if business data is different
            // Get current state at execution time to avoid dependency issues
            const currentHerdModules = store.getState().scout.herd_modules;
            const herdModulesChanged = conditionalDispatch(
              cachedHerdModules,
              currentHerdModules,
              setHerdModules,
              "Herd modules (cache)",
              true, // Skip timestamp-only updates
            );

            if (herdModulesChanged) {
              dispatch(
                setHerdModulesLoadingState(
                  EnumHerdModulesLoadingState.SUCCESSFULLY_LOADED,
                ),
              );
            }

            // If cache is fresh, we still background fetch but don't wait
            if (!cacheResult.isStale) {
              // Background fetch fresh data without blocking
              (async () => {
                try {
                  const backgroundStartTime = Date.now();

                  const [backgroundHerdModulesResult, backgroundUserResult] =
                    await Promise.all([
                      (async () => {
                        const start = Date.now();
                        const result = await server_load_herd_modules();
                        const totalDuration = Date.now() - start;
                        const serverDuration =
                          result.server_processing_time_ms || totalDuration;
                        const clientOverhead = totalDuration - serverDuration;

                        console.log(
                          `[useScoutRefresh] Background API timing breakdown:`,
                        );
                        console.log(
                          `  - Server processing: ${serverDuration}ms`,
                        );
                        console.log(`  - Client overhead: ${clientOverhead}ms`);
                        console.log(`  - Total request: ${totalDuration}ms`);

                        timingRefs.current.herdModulesDuration = serverDuration;
                        dispatch(
                          setHerdModulesApiServerProcessingDuration(
                            serverDuration,
                          ),
                        );
                        dispatch(
                          setHerdModulesApiTotalRequestDuration(totalDuration),
                        );
                        return result;
                      })(),
                      (async () => {
                        const start = Date.now();
                        const { data } = await supabase.auth.getUser();
                        const duration = Date.now() - start;
                        timingRefs.current.userApiDuration = duration;
                        dispatch(setUserApiDuration(duration));
                        return { data: data.user, status: "success" };
                      })(),
                    ]);

                  const backgroundDuration = Date.now() - backgroundStartTime;

                  // Validate background responses
                  if (
                    backgroundHerdModulesResult.data &&
                    Array.isArray(backgroundHerdModulesResult.data) &&
                    backgroundUserResult &&
                    backgroundUserResult.data
                  ) {
                    // Update cache with fresh data
                    try {
                      await scoutCache.setHerdModules(
                        backgroundHerdModulesResult.data,
                        cacheTtlMs,
                      );
                    } catch (cacheError) {
                      console.warn(
                        "[useScoutRefresh] Background cache save failed:",
                        cacheError,
                      );
                      await handleIndexedDbError(
                        cacheError,
                        "background cache save",
                        async () => {
                          if (backgroundHerdModulesResult.data) {
                            await scoutCache.setHerdModules(
                              backgroundHerdModulesResult.data,
                              cacheTtlMs,
                            );
                          }
                        },
                      );
                    }

                    // Conditionally update store with fresh background data, skip timestamp-only changes
                    const currentHerdModules =
                      store.getState().scout.herd_modules;
                    const currentUser = store.getState().scout.user;

                    conditionalDispatch(
                      backgroundHerdModulesResult.data,
                      currentHerdModules,
                      setHerdModules,
                      "Herd modules (background)",
                      true, // Skip timestamp-only updates
                    );

                    if (backgroundUserResult && backgroundUserResult.data) {
                      conditionalDispatch(
                        backgroundUserResult.data,
                        currentUser,
                        setUser,
                        "User (background)",
                      );
                    }

                    // Update data source to DATABASE
                    dispatch(setDataSource(EnumDataSource.DATABASE));
                    dispatch(
                      setDataSourceInfo({
                        source: EnumDataSource.DATABASE,
                        timestamp: Date.now(),
                      }),
                    );
                  } else {
                    console.warn(
                      "[useScoutRefresh] Background fetch returned invalid data",
                    );
                  }
                } catch (backgroundError) {
                  console.warn(
                    "[useScoutRefresh] Background fetch failed:",
                    backgroundError,
                  );
                }
              })();

              const totalDuration = Date.now() - startTime;
              dispatch(setHerdModulesLoadedInMs(totalDuration));
              dispatch(setStatus(EnumScoutStateStatus.DONE_LOADING));

              onRefreshComplete?.();
              return;
            }
          } else {
          }
        } catch (cacheError) {
          console.warn("[useScoutRefresh] Cache load failed:", cacheError);
          await handleIndexedDbError(cacheError, "cache load");
          // Continue with API call
        }
      }

      // Step 2: Load fresh data from API
      const parallelStartTime = Date.now();

      const [herdModulesResult, userResult] = await Promise.all([
        (async () => {
          const start = Date.now();

          const result = await server_load_herd_modules();
          const totalDuration = Date.now() - start;
          const serverDuration =
            result.server_processing_time_ms || totalDuration;

          return { result, totalDuration, serverDuration, start };
        })(),
        (async () => {
          const start = Date.now();

          const { data } = await supabase.auth.getUser();
          const duration = Date.now() - start;

          return {
            result: { data: data.user, status: "success" },
            duration,
            start,
          };
        })(),
      ]);

      const parallelDuration = Date.now() - parallelStartTime;
      console.log(
        `[useScoutRefresh] Parallel API requests completed in ${parallelDuration}ms`,
      );

      // Extract results and timing
      const herdModulesResponse = herdModulesResult.result;
      const res_new_user = userResult.result;
      const herdModulesServerDuration = herdModulesResult.serverDuration;
      const herdModulesTotalDuration = herdModulesResult.totalDuration;
      const userApiDuration = userResult.duration;
      const clientOverhead =
        herdModulesTotalDuration - herdModulesServerDuration;

      console.log(`[useScoutRefresh] Fresh API timing breakdown:`);
      console.log(`  - Server processing: ${herdModulesServerDuration}ms`);
      console.log(`  - Client overhead: ${clientOverhead}ms`);
      console.log(`  - Total request: ${herdModulesTotalDuration}ms`);

      // Store timing values
      timingRefs.current.herdModulesDuration = herdModulesServerDuration;
      timingRefs.current.userApiDuration = userApiDuration;

      // Dispatch timing actions
      dispatch(
        setHerdModulesApiServerProcessingDuration(herdModulesServerDuration),
      );
      dispatch(setHerdModulesApiTotalRequestDuration(herdModulesTotalDuration));
      dispatch(setUserApiDuration(userApiDuration));

      // Validate API responses
      const validationStartTime = Date.now();
      if (
        !herdModulesResponse.data ||
        !Array.isArray(herdModulesResponse.data)
      ) {
        throw new Error("Invalid herd modules response");
      }

      if (!res_new_user || !res_new_user.data) {
        throw new Error("Invalid user response");
      }
      const validationDuration = Date.now() - validationStartTime;
      console.log(
        `[useScoutRefresh] Data validation took: ${validationDuration}ms`,
      );

      // Use the validated data
      const compatible_new_herd_modules = herdModulesResponse.data;

      // Set data source to DATABASE
      dispatch(setDataSource(EnumDataSource.DATABASE));
      dispatch(
        setDataSourceInfo({
          source: EnumDataSource.DATABASE,
          timestamp: Date.now(),
        }),
      );

      // Step 3: Update cache with fresh data
      const cacheSaveStartTime = Date.now();
      try {
        await scoutCache.setHerdModules(
          compatible_new_herd_modules,
          cacheTtlMs,
        );
        const cacheSaveDuration = Date.now() - cacheSaveStartTime;
        timingRefs.current.cacheSaveDuration = cacheSaveDuration;
        console.log(
          `[useScoutRefresh] Cache updated in ${cacheSaveDuration}ms with TTL: ${Math.round(
            cacheTtlMs / 1000,
          )}s`,
        );
      } catch (cacheError) {
        console.warn("[useScoutRefresh] Cache save failed:", cacheError);
        await handleIndexedDbError(cacheError, "cache save", async () => {
          await scoutCache.setHerdModules(
            compatible_new_herd_modules,
            cacheTtlMs,
          );
        });
      }

      // Step 4: Conditionally update store with fresh data, skip timestamp-only changes
      const dataProcessingStartTime = Date.now();

      const currentHerdModules = store.getState().scout.herd_modules;
      const currentUser = store.getState().scout.user;

      const herdModulesChanged = conditionalDispatch(
        compatible_new_herd_modules,
        currentHerdModules,
        setHerdModules,
        "Herd modules (fresh API)",
        true, // Skip timestamp-only updates
      );

      const userChanged = conditionalDispatch(
        res_new_user.data,
        currentUser,
        setUser,
        "User (fresh API)",
      );

      if (herdModulesChanged) {
        dispatch(
          setHerdModulesLoadingState(
            EnumHerdModulesLoadingState.SUCCESSFULLY_LOADED,
          ),
        );
      }

      const dataProcessingDuration = Date.now() - dataProcessingStartTime;
      timingRefs.current.dataProcessingDuration = dataProcessingDuration;
      dispatch(setDataProcessingDuration(dataProcessingDuration));

      const loadingDuration = Date.now() - startTime;
      dispatch(setHerdModulesLoadedInMs(loadingDuration));

      dispatch(setStatus(EnumScoutStateStatus.DONE_LOADING));

      // Log concise completion summary
      console.log(
        `[useScoutRefresh] Refresh completed in ${loadingDuration}ms (Server: ${herdModulesServerDuration}ms, Total API: ${herdModulesTotalDuration}ms)`,
      );

      onRefreshComplete?.();
    } catch (error) {
      const loadingDuration = Date.now() - startTime;
      console.error("Error refreshing scout data:", error);

      // Set data source to UNKNOWN on error
      dispatch(setDataSource(EnumDataSource.UNKNOWN));
      dispatch(
        setDataSourceInfo({
          source: EnumDataSource.UNKNOWN,
          timestamp: Date.now(),
        }),
      );

      // Ensure consistent state updates on error
      dispatch(
        setHerdModulesLoadingState(
          EnumHerdModulesLoadingState.UNSUCCESSFULLY_LOADED,
        ),
      );
      dispatch(setHerdModulesLoadedInMs(loadingDuration));
      dispatch(setStatus(EnumScoutStateStatus.DONE_LOADING));

      // Log essential error metrics
      console.log(
        `[useScoutRefresh] Refresh failed after ${loadingDuration}ms`,
      );

      // Call completion callback even on error for consistency
      onRefreshComplete?.();
    } finally {
      refreshInProgressRef.current = false;
    }
  }, [
    dispatch,
    store,
    supabase,
    onRefreshComplete,
    cacheFirst,
    cacheTtlMs,
    conditionalDispatch,
    handleIndexedDbError,
  ]);

  useEffect(() => {
    if (autoRefresh) {
      handleRefresh();
    }
  }, [autoRefresh, handleRefresh]);

  // Utility function to clear cache
  const clearCache = useCallback(async () => {
    try {
      await scoutCache.clearHerdModules();
    } catch (error) {
      console.error("[useScoutRefresh] Failed to clear cache:", error);
    }
  }, []);

  return {
    handleRefresh,
    clearCache,
  };
}
