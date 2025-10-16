import { useEffect, useCallback, useRef } from "react";
import { useAppDispatch } from "../store/hooks";
import {
  EnumScoutStateStatus,
  setActiveHerdId,
  setHerdModules,
  setStatus,
  setHerdModulesLoadingState,
  setHerdModulesLoadedInMs,
  setHerdModulesApiDuration,
  setUserApiDuration,
  setDataProcessingDuration,
  setLocalStorageDuration,
  setUser,
  setDataSource,
  setDataSourceInfo,
} from "../store/scout";
import { EnumHerdModulesLoadingState } from "../types/herd_module";
import { server_load_herd_modules } from "../helpers/herds";
import { server_get_user } from "../helpers/users";
import {
  scoutCache,
  CacheStats,
  TimingStats,
  DatabaseHealth,
} from "../helpers/cache";
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
 * - getTimingStats: Function to get detailed timing statistics for the last refresh
 * - clearCache: Function to clear the cache
 * - getCacheStats: Function to get cache statistics
 *
 * @example
 * ```tsx
 * const { handleRefresh, getTimingStats, clearCache, getCacheStats } = useScoutRefresh({
 *   cacheFirst: true,
 *   cacheTtlMs: 10 * 60 * 1000 // 10 minutes
 * });
 *
 * // Get timing stats after a refresh
 * const stats = getTimingStats();
 * console.log('Herd modules API took:', stats.herdModulesApi, 'ms');
 * console.log('User API took:', stats.userApi, 'ms');
 * console.log('Data processing took:', stats.dataProcessing, 'ms');
 * console.log('LocalStorage operations took:', stats.localStorage, 'ms');
 * console.log('Total duration:', stats.totalDuration, 'ms');
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
  const refreshInProgressRef = useRef(false);

  // Refs to store timing measurements
  const timingRefs = useRef({
    startTime: 0,
    herdModulesDuration: 0,
    userApiDuration: 0,
    dataProcessingDuration: 0,
    localStorageDuration: 0,
    cacheLoadDuration: 0,
    cacheSaveDuration: 0,
  });

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

            // Immediately update the store with cached data
            dispatch(setHerdModules(cachedHerdModules));
            dispatch(
              setHerdModulesLoadingState(
                EnumHerdModulesLoadingState.SUCCESSFULLY_LOADED,
              ),
            );

            // Always load user data from API
            const userStartTime = Date.now();
            const res_new_user = await server_get_user();
            const userApiDuration = Date.now() - userStartTime;
            timingRefs.current.userApiDuration = userApiDuration;
            dispatch(setUserApiDuration(userApiDuration));

            if (res_new_user && res_new_user.data) {
              dispatch(setUser(res_new_user.data));
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

                      // If it's an IndexedDB object store error, try to reset the database
                      if (
                        cacheError instanceof Error &&
                        (cacheError.message.includes("object store") ||
                          cacheError.message.includes("NotFoundError"))
                      ) {
                        console.log(
                          "[useScoutRefresh] Attempting database reset due to schema error...",
                        );
                        try {
                          await scoutCache.resetDatabase();
                          console.log(
                            "[useScoutRefresh] Database reset successful, retrying cache save...",
                          );
                          await scoutCache.setHerdModules(
                            backgroundHerdModulesResult.data,
                            cacheTtlMs,
                          );
                          console.log(
                            "[useScoutRefresh] Cache save successful after database reset",
                          );
                        } catch (resetError) {
                          console.error(
                            "[useScoutRefresh] Database reset and retry failed:",
                            resetError,
                          );
                        }
                      }
                    }

                    // Update store with fresh data
                    dispatch(setHerdModules(backgroundHerdModulesResult.data));
                    dispatch(setUser(backgroundUserResult.data));

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

          // If it's an IndexedDB object store error, try to reset the database
          if (
            cacheError instanceof Error &&
            (cacheError.message.includes("object store") ||
              cacheError.message.includes("NotFoundError"))
          ) {
            console.log(
              "[useScoutRefresh] Attempting database reset due to cache load error...",
            );
            try {
              await scoutCache.resetDatabase();
              console.log("[useScoutRefresh] Database reset successful");
            } catch (resetError) {
              console.error(
                "[useScoutRefresh] Database reset failed:",
                resetError,
              );
            }
          }
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

        // If it's an IndexedDB object store error, try to reset the database
        if (
          cacheError instanceof Error &&
          (cacheError.message.includes("object store") ||
            cacheError.message.includes("NotFoundError"))
        ) {
          console.log(
            "[useScoutRefresh] Attempting database reset due to cache save error...",
          );
          try {
            await scoutCache.resetDatabase();
            console.log(
              "[useScoutRefresh] Database reset successful, retrying cache save...",
            );
            await scoutCache.setHerdModules(
              compatible_new_herd_modules,
              cacheTtlMs,
            );
            console.log(
              "[useScoutRefresh] Cache save successful after database reset",
            );
          } catch (resetError) {
            console.error(
              "[useScoutRefresh] Database reset and retry failed:",
              resetError,
            );
          }
        }
      }

      // Step 4: Update store with fresh data
      const dataProcessingStartTime = Date.now();

      dispatch(setHerdModules(compatible_new_herd_modules));
      dispatch(setUser(res_new_user.data));
      dispatch(
        setHerdModulesLoadingState(
          EnumHerdModulesLoadingState.SUCCESSFULLY_LOADED,
        ),
      );

      const dataProcessingDuration = Date.now() - dataProcessingStartTime;
      timingRefs.current.dataProcessingDuration = dataProcessingDuration;
      dispatch(setDataProcessingDuration(dataProcessingDuration));

      // Step 5: Handle localStorage operations
      const localStorageStartTime = Date.now();

      try {
        // Check local storage for a last selected herd
        const lastSelectedHerd = localStorage.getItem("last_selected_herd");
        if (lastSelectedHerd) {
          const found_herd = compatible_new_herd_modules.find(
            (hm) => hm.herd.id.toString() === lastSelectedHerd,
          )?.herd;

          // If herd is found then set it
          if (found_herd) {
            dispatch(setActiveHerdId(found_herd.id.toString()));
          }
        }
        // If there is no last selected herd then select the first one
        else if (compatible_new_herd_modules.length > 0) {
          const firstHerdId = compatible_new_herd_modules[0].herd.id.toString();
          localStorage.setItem("last_selected_herd", firstHerdId);
          dispatch(setActiveHerdId(firstHerdId));
        }
      } catch (localStorageError) {
        console.warn(
          "[useScoutRefresh] localStorage not available:",
          localStorageError,
        );
        // Fallback: select first herd without localStorage
        if (compatible_new_herd_modules.length > 0) {
          dispatch(
            setActiveHerdId(compatible_new_herd_modules[0].herd.id.toString()),
          );
        }
      }

      const localStorageDuration = Date.now() - localStorageStartTime;
      timingRefs.current.localStorageDuration = localStorageDuration;
      dispatch(setLocalStorageDuration(localStorageDuration));

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
      console.log(`  - LocalStorage: ${localStorageDuration}ms`);
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
  }, [dispatch, onRefreshComplete, cacheFirst, cacheTtlMs]);

  useEffect(() => {
    if (autoRefresh) {
      handleRefresh();
    }
  }, [autoRefresh, handleRefresh]);

  // Utility function to get timing statistics
  const getTimingStats = useCallback((): TimingStats => {
    const now = Date.now();
    const startTime = timingRefs.current.startTime;
    return {
      totalDuration: startTime > 0 ? now - startTime : 0,
      cacheLoad: timingRefs.current.cacheLoadDuration,
      herdModulesApi: timingRefs.current.herdModulesDuration,
      userApi: timingRefs.current.userApiDuration,
      cacheSave: timingRefs.current.cacheSaveDuration,
      dataProcessing: timingRefs.current.dataProcessingDuration,
      localStorage: timingRefs.current.localStorageDuration,
    };
  }, []);

  // Utility function to clear cache
  const clearCache = useCallback(async () => {
    try {
      await scoutCache.clearHerdModules();
      console.log("[useScoutRefresh] Cache cleared successfully");
    } catch (error) {
      console.error("[useScoutRefresh] Failed to clear cache:", error);
    }
  }, []);

  // Utility function to get cache statistics
  const getCacheStats = useCallback(async (): Promise<CacheStats> => {
    try {
      return await scoutCache.getCacheStats();
    } catch (error) {
      console.error("[useScoutRefresh] Failed to get cache stats:", error);
      return {
        size: 0,
        lastUpdated: 0,
        isStale: true,
        hitRate: 0,
        totalHits: 0,
        totalMisses: 0,
      };
    }
  }, []);

  // Utility function to check database health
  const checkDatabaseHealth = useCallback(async () => {
    try {
      return await scoutCache.checkDatabaseHealth();
    } catch (error) {
      console.error(
        "[useScoutRefresh] Failed to check database health:",
        error,
      );
      return {
        healthy: false,
        issues: [`Health check failed: ${error}`],
      };
    }
  }, []);

  // Utility function to reset database
  const resetDatabase = useCallback(async () => {
    try {
      await scoutCache.resetDatabase();
      console.log("[useScoutRefresh] Database reset successfully");
    } catch (error) {
      console.error("[useScoutRefresh] Failed to reset database:", error);
      throw error;
    }
  }, []);

  return {
    handleRefresh,
    getTimingStats,
    clearCache,
    getCacheStats,
    checkDatabaseHealth,
    resetDatabase,
  };
}
