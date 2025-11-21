import { useEffect, useCallback, useRef } from "react";
import { useAppDispatch, useHerdModules, useUser } from "../store/hooks";
import {
  EnumScoutStateStatus,
  setHerdModules,
  setStatus,
  setHerdModulesLoadingState,
  setHerdModulesLoadedInMs,
  setHerdModulesApiDuration,
  setUserApiDuration,
  setDataProcessingDuration,
  setUser,
  setDataSource,
  setDataSourceInfo,
} from "../store/scout";
import { EnumHerdModulesLoadingState } from "../types/herd_module";
import { server_load_herd_modules } from "../helpers/herds";
import { server_get_user } from "../helpers/users";
import { scoutCache } from "../helpers/cache";
import { EnumDataSource } from "../types/data_source";

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
  const currentHerdModules = useHerdModules();
  const currentUser = useUser();
  const refreshInProgressRef = useRef(false);

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
  const deepEqual = useCallback((obj1: any, obj2: any): boolean => {
    if (obj1 === obj2) return true;

    if (obj1 == null || obj2 == null) return obj1 === obj2;

    if (typeof obj1 !== typeof obj2) return false;

    if (typeof obj1 !== "object") return obj1 === obj2;

    if (Array.isArray(obj1) !== Array.isArray(obj2)) return false;

    if (Array.isArray(obj1)) {
      if (obj1.length !== obj2.length) return false;
      for (let i = 0; i < obj1.length; i++) {
        if (!deepEqual(obj1[i], obj2[i])) return false;
      }
      return true;
    }

    const keys1 = Object.keys(obj1);
    const keys2 = Object.keys(obj2);

    if (keys1.length !== keys2.length) return false;

    for (const key of keys1) {
      if (!keys2.includes(key)) return false;
      if (!deepEqual(obj1[key], obj2[key])) return false;
    }

    return true;
  }, []);

  // Helper function to conditionally dispatch only if data has changed
  const conditionalDispatch = useCallback(
    (
      newData: any,
      currentData: any,
      actionCreator: (data: any) => any,
      dataType: string,
    ) => {
      if (!deepEqual(newData, currentData)) {
        console.log(
          `[useScoutRefresh] ${dataType} data changed, updating store`,
        );
        dispatch(actionCreator(newData));
        return true;
      } else {
        console.log(
          `[useScoutRefresh] ${dataType} data unchanged, skipping store update`,
        );
        return false;
      }
    },
    [dispatch, deepEqual],
  );

  // Helper function to handle IndexedDB errors
  const handleIndexedDbError = async (
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
  };

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
          console.log("[useScoutRefresh] Loading from cache...");
          const cacheResult = await scoutCache.getHerdModules();
          cacheLoadDuration = Date.now() - cacheStartTime;
          timingRefs.current.cacheLoadDuration = cacheLoadDuration;

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

            // Conditionally update the store with cached data if different
            const herdModulesChanged = conditionalDispatch(
              cachedHerdModules,
              currentHerdModules,
              setHerdModules,
              "Herd modules (cache)",
            );

            if (herdModulesChanged) {
              dispatch(
                setHerdModulesLoadingState(
                  EnumHerdModulesLoadingState.SUCCESSFULLY_LOADED,
                ),
              );
            }

            // Always load user data from API
            const userStartTime = Date.now();
            const res_new_user = await server_get_user();
            const userApiDuration = Date.now() - userStartTime;
            timingRefs.current.userApiDuration = userApiDuration;
            dispatch(setUserApiDuration(userApiDuration));

            if (res_new_user && res_new_user.data) {
              conditionalDispatch(
                res_new_user.data,
                currentUser,
                setUser,
                "User (initial)",
              );
            }

            // If cache is fresh, we still background fetch but don't wait
            if (!cacheResult.isStale) {
              console.log(
                "[useScoutRefresh] Cache is fresh, background fetching fresh data...",
              );

              // Background fetch fresh data without blocking
              (async () => {
                try {
                  console.log("[useScoutRefresh] Starting background fetch...");
                  const backgroundStartTime = Date.now();

                  const [backgroundHerdModulesResult, backgroundUserResult] =
                    await Promise.all([
                      server_load_herd_modules(),
                      server_get_user(),
                    ]);

                  const backgroundDuration = Date.now() - backgroundStartTime;
                  console.log(
                    `[useScoutRefresh] Background fetch completed in ${backgroundDuration}ms`,
                  );

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
                      console.log(
                        `[useScoutRefresh] Background cache updated with TTL: ${Math.round(
                          cacheTtlMs / 1000,
                        )}s`,
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

                    // Conditionally update store with fresh background data
                    conditionalDispatch(
                      backgroundHerdModulesResult.data,
                      currentHerdModules,
                      setHerdModules,
                      "Herd modules (background)",
                    );
                    conditionalDispatch(
                      backgroundUserResult.data,
                      currentUser,
                      setUser,
                      "User (background)",
                    );

                    // Update data source to DATABASE
                    dispatch(setDataSource(EnumDataSource.DATABASE));
                    dispatch(
                      setDataSourceInfo({
                        source: EnumDataSource.DATABASE,
                        timestamp: Date.now(),
                      }),
                    );

                    console.log(
                      "[useScoutRefresh] Background fetch completed and store updated",
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

              console.log(
                `[useScoutRefresh] Cache-first refresh completed in ${totalDuration}ms (background fetch in progress)`,
              );
              onRefreshComplete?.();
              return;
            }
          } else {
            console.log("[useScoutRefresh] No cached data found");
          }
        } catch (cacheError) {
          console.warn("[useScoutRefresh] Cache load failed:", cacheError);
          await handleIndexedDbError(cacheError, "cache load");
          // Continue with API call
        }
      }

      // Step 2: Load fresh data from API
      console.log("[useScoutRefresh] Loading fresh data from API...");
      const parallelStartTime = Date.now();

      const [herdModulesResult, userResult] = await Promise.all([
        (async () => {
          const start = Date.now();
          console.log(
            `[useScoutRefresh] Starting herd modules request at ${new Date(
              start,
            ).toISOString()}`,
          );

          const result = await server_load_herd_modules();
          const duration = Date.now() - start;
          console.log(
            `[useScoutRefresh] Herd modules request completed in ${duration}ms`,
          );
          return { result, duration, start };
        })(),
        (async () => {
          const start = Date.now();
          console.log(
            `[useScoutRefresh] Starting user request at ${new Date(
              start,
            ).toISOString()}`,
          );

          const result = await server_get_user();
          const duration = Date.now() - start;
          console.log(
            `[useScoutRefresh] User request completed in ${duration}ms`,
          );
          return { result, duration, start };
        })(),
      ]);

      const parallelDuration = Date.now() - parallelStartTime;
      console.log(
        `[useScoutRefresh] Parallel API requests completed in ${parallelDuration}ms`,
      );

      // Extract results and timing
      const herdModulesResponse = herdModulesResult.result;
      const res_new_user = userResult.result;
      const herdModulesDuration = herdModulesResult.duration;
      const userApiDuration = userResult.duration;

      // Store timing values
      timingRefs.current.herdModulesDuration = herdModulesDuration;
      timingRefs.current.userApiDuration = userApiDuration;

      // Dispatch timing actions
      dispatch(setHerdModulesApiDuration(herdModulesDuration));
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

      // Step 4: Conditionally update store with fresh data if different
      const dataProcessingStartTime = Date.now();

      const herdModulesChanged = conditionalDispatch(
        compatible_new_herd_modules,
        currentHerdModules,
        setHerdModules,
        "Herd modules (fresh API)",
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

      // Log essential performance metrics
      console.log(`[useScoutRefresh] Refresh completed successfully:`);
      console.log(`  - Total duration: ${loadingDuration}ms`);
      console.log(`  - Cache load: ${cacheLoadDuration}ms`);
      console.log(`  - Herd modules API: ${herdModulesDuration}ms`);
      console.log(`  - User API: ${userApiDuration}ms`);
      console.log(`  - Cache save: ${timingRefs.current.cacheSaveDuration}ms`);
      console.log(`  - Data processing: ${dataProcessingDuration}ms`);
      console.log(`  - Cache TTL: ${Math.round(cacheTtlMs / 1000)}s`);

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
      console.log(`[useScoutRefresh] Refresh failed:`);
      console.log(`  - Total duration: ${loadingDuration}ms`);
      console.log(
        `  - Herd modules: ${timingRefs.current.herdModulesDuration}ms`,
      );
      console.log(`  - User API: ${timingRefs.current.userApiDuration}ms`);
    } finally {
      refreshInProgressRef.current = false;
    }
  }, [
    dispatch,
    onRefreshComplete,
    cacheFirst,
    cacheTtlMs,
    handleIndexedDbError,
    currentHerdModules,
    currentUser,
    conditionalDispatch,
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
      console.log("[useScoutRefresh] Cache cleared successfully");
    } catch (error) {
      console.error("[useScoutRefresh] Failed to clear cache:", error);
    }
  }, []);

  return {
    handleRefresh,
    clearCache,
  };
}
