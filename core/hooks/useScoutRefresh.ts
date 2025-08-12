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
} from "../store/scout";
import { EnumHerdModulesLoadingState } from "../types/herd_module";
import { server_load_herd_modules } from "../helpers/herds";
import { server_get_user } from "../helpers/users";
import { EnumWebResponse } from "../types/requests";

export interface UseScoutRefreshOptions {
  autoRefresh?: boolean;
  onRefreshComplete?: () => void;
}

/**
 * Hook for refreshing scout data with detailed timing measurements
 *
 * @param options - Configuration options for the refresh behavior
 * @param options.autoRefresh - Whether to automatically refresh on mount (default: true)
 * @param options.onRefreshComplete - Callback function called when refresh completes
 *
 * @returns Object containing:
 * - handleRefresh: Function to manually trigger a refresh
 * - getTimingStats: Function to get detailed timing statistics for the last refresh
 *
 * @example
 * ```tsx
 * const { handleRefresh, getTimingStats } = useScoutRefresh();
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
  const { autoRefresh = true, onRefreshComplete } = options;
  const dispatch = useAppDispatch();
  const refreshInProgressRef = useRef(false);

  // Refs to store timing measurements
  const timingRefs = useRef({
    startTime: 0,
    herdModulesDuration: 0,
    userApiDuration: 0,
    dataProcessingDuration: 0,
    localStorageDuration: 0,
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

      // Run API requests in parallel for better performance
      console.log("[useScoutRefresh] Starting parallel API requests...");
      const parallelStartTime = Date.now();

      const [herdModulesResult, userResult] = await Promise.all([
        (async () => {
          const start = Date.now();
          console.log(
            `[useScoutRefresh] Starting herd modules request at ${new Date(
              start
            ).toISOString()}`
          );

          // High priority request with optimization
          const result = await server_load_herd_modules();
          const duration = Date.now() - start;
          console.log(
            `[useScoutRefresh] Herd modules request completed in ${duration}ms`
          );
          return { result, duration, start };
        })(),
        (async () => {
          const start = Date.now();
          console.log(
            `[useScoutRefresh] Starting user request at ${new Date(
              start
            ).toISOString()}`
          );

          // High priority request with optimization
          const result = await server_get_user();
          const duration = Date.now() - start;
          console.log(
            `[useScoutRefresh] User request completed in ${duration}ms`
          );
          return { result, duration, start };
        })(),
      ]);

      const parallelDuration = Date.now() - parallelStartTime;
      console.log(
        `[useScoutRefresh] Parallel API requests completed in ${parallelDuration}ms`
      );

      // Extract results and timing
      const herdModulesResponse = herdModulesResult.result;
      const res_new_user = userResult.result;
      const herdModulesDuration = herdModulesResult.duration;
      const userApiDuration = userResult.duration;

      // Calculate request timing breakdown
      const requestStartTime = parallelStartTime;
      const requestEndTime = Date.now();
      const totalRequestTime = requestEndTime - requestStartTime;

      console.log(`[useScoutRefresh] Request timing breakdown:`);
      console.log(
        `  - Request started at: ${new Date(requestStartTime).toISOString()}`
      );
      console.log(
        `  - Request completed at: ${new Date(requestEndTime).toISOString()}`
      );
      console.log(`  - Total request time: ${totalRequestTime}ms`);
      console.log(`  - Parallel execution time: ${parallelDuration}ms`);
      console.log(
        `  - Request overhead: ${totalRequestTime - parallelDuration}ms`
      );

      // Calculate network latency for herd modules
      let networkLatencyMs = 0;
      if (
        herdModulesResponse.status === EnumWebResponse.SUCCESS &&
        herdModulesResponse.data
      ) {
        const serverFinishTime = herdModulesResponse.time_finished;
        const clientReceiveTime = Date.now();
        const estimatedNetworkLatency = clientReceiveTime - serverFinishTime;
        networkLatencyMs = Math.max(0, estimatedNetworkLatency);

        console.log(`[useScoutRefresh] Herd modules performance:`);
        console.log(
          `  - Server processing: ${herdModulesResponse.server_processing_time_ms}ms`
        );
        console.log(`  - Network latency: ${networkLatencyMs}ms`);
        console.log(`  - Total client time: ${herdModulesDuration}ms`);
      }

      // Store timing values
      timingRefs.current.herdModulesDuration = herdModulesDuration;
      timingRefs.current.userApiDuration = userApiDuration;

      // Dispatch timing actions
      dispatch(setHerdModulesApiDuration(herdModulesDuration));
      dispatch(setUserApiDuration(userApiDuration));

      // Calculate network overhead
      const totalApiTime = herdModulesDuration + userApiDuration;
      const networkOverhead =
        parallelDuration - Math.max(herdModulesDuration, userApiDuration);

      console.log(`[useScoutRefresh] API performance:`);
      console.log(`  - Herd modules: ${herdModulesDuration}ms`);
      console.log(`  - User API: ${userApiDuration}ms`);
      console.log(`  - Parallel execution: ${parallelDuration}ms`);
      console.log(
        `  - Time saved with parallel: ${totalApiTime - parallelDuration}ms`
      );

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
        `[useScoutRefresh] Data validation took: ${validationDuration}ms`
      );

      // Use the validated data
      const compatible_new_herd_modules = herdModulesResponse.data;

      // Measure data processing duration
      const dataProcessingStartTime = Date.now();

      dispatch(setHerdModules(compatible_new_herd_modules));
      dispatch(setUser(res_new_user.data));
      dispatch(
        setHerdModulesLoadingState(
          EnumHerdModulesLoadingState.SUCCESSFULLY_LOADED
        )
      );

      const dataProcessingDuration = Date.now() - dataProcessingStartTime;
      timingRefs.current.dataProcessingDuration = dataProcessingDuration;
      dispatch(setDataProcessingDuration(dataProcessingDuration));

      // Measure localStorage operations duration
      const localStorageStartTime = Date.now();

      // Safely handle localStorage operations
      try {
        // Check local storage for a last selected herd
        const lastSelectedHerd = localStorage.getItem("last_selected_herd");
        if (lastSelectedHerd) {
          const found_herd = compatible_new_herd_modules.find(
            (hm) => hm.herd.id.toString() === lastSelectedHerd
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
          localStorageError
        );
        // Fallback: select first herd without localStorage
        if (compatible_new_herd_modules.length > 0) {
          dispatch(
            setActiveHerdId(compatible_new_herd_modules[0].herd.id.toString())
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
      console.log(`  - Herd modules: ${herdModulesDuration}ms`);
      console.log(`  - User API: ${userApiDuration}ms`);
      console.log(`  - Parallel execution: ${parallelDuration}ms`);
      console.log(
        `  - Time saved with parallel: ${totalApiTime - parallelDuration}ms`
      );

      onRefreshComplete?.();
    } catch (error) {
      const loadingDuration = Date.now() - startTime;
      console.error("Error refreshing scout data:", error);

      // Ensure consistent state updates on error
      dispatch(
        setHerdModulesLoadingState(
          EnumHerdModulesLoadingState.UNSUCCESSFULLY_LOADED
        )
      );
      dispatch(setHerdModulesLoadedInMs(loadingDuration));
      dispatch(setStatus(EnumScoutStateStatus.DONE_LOADING));

      // Log essential error metrics
      console.log(`[useScoutRefresh] Refresh failed:`);
      console.log(`  - Total duration: ${loadingDuration}ms`);
      console.log(
        `  - Herd modules: ${timingRefs.current.herdModulesDuration}ms`
      );
      console.log(`  - User API: ${timingRefs.current.userApiDuration}ms`);
    } finally {
      refreshInProgressRef.current = false;
    }
  }, [dispatch, onRefreshComplete]);

  useEffect(() => {
    if (autoRefresh) {
      handleRefresh();
    }
  }, [autoRefresh, handleRefresh]);

  // Utility function to get timing statistics
  const getTimingStats = useCallback(() => {
    const now = Date.now();
    const startTime = timingRefs.current.startTime;
    return {
      totalDuration: startTime > 0 ? now - startTime : 0,
      herdModulesApi: timingRefs.current.herdModulesDuration,
      userApi: timingRefs.current.userApiDuration,
      dataProcessing: timingRefs.current.dataProcessingDuration,
      localStorage: timingRefs.current.localStorageDuration,
    };
  }, []);

  return {
    handleRefresh,
    getTimingStats,
  };
}
