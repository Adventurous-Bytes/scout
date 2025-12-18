import { useDispatch, useSelector } from "react-redux";
import { EnumHerdModulesLoadingState } from "../types/herd_module";
import { LoadingPerformance } from "./scout";
import { ISessionSummary } from "../types/db";

// Simple wrapper for useDispatch to maintain compatibility
export const useAppDispatch = useDispatch;

// Selector hook for herd modules loading state
export const useHerdModulesLoadingState = () => {
  return useSelector((state: any) => state.scout.herd_modules_loading_state);
};

// Selector hook for checking if herd modules are currently loading
export const useIsHerdModulesLoading = () => {
  return useSelector(
    (state: any) =>
      state.scout.herd_modules_loading_state ===
      EnumHerdModulesLoadingState.LOADING,
  );
};

// Selector hook for checking if herd modules loaded successfully
export const useIsHerdModulesLoaded = () => {
  return useSelector(
    (state: any) =>
      state.scout.herd_modules_loading_state ===
      EnumHerdModulesLoadingState.SUCCESSFULLY_LOADED,
  );
};

// Selector hook for checking if herd modules failed to load
export const useIsHerdModulesFailed = () => {
  return useSelector(
    (state: any) =>
      state.scout.herd_modules_loading_state ===
      EnumHerdModulesLoadingState.UNSUCCESSFULLY_LOADED,
  );
};

// Selector hook for getting when herd modules were last loaded
export const useHerdModulesLoadedAt = () => {
  return useSelector(
    (state: any) => state.scout.loading_performance.herd_modules_loaded_in_ms,
  );
};

// Selector hook for getting the loading duration in milliseconds
export const useHerdModulesLoadingDuration = () => {
  return useSelector((state: any) => {
    return state.scout.loading_performance.herd_modules_loaded_in_ms;
  });
};

// Selector hook for getting the complete loading performance object
export const useLoadingPerformance = (): LoadingPerformance => {
  return useSelector((state: any) => state.scout.loading_performance);
};

// Selector hook for getting session summaries for a specific herd
export const useSessionSummariesByHerd = (
  herdId: number,
): ISessionSummary | null => {
  return useSelector((state: any) => {
    const herdModule = state.scout.herd_modules.find(
      (hm: any) => hm.herd.id === herdId,
    );
    return herdModule?.session_summaries || null;
  });
};

// Selector hook for checking if session summaries are available for a herd
export const useHasSessionSummaries = (herdId: number): boolean => {
  return useSelector((state: any) => {
    const herdModule = state.scout.herd_modules.find(
      (hm: any) => hm.herd.id === herdId,
    );
    return !!herdModule?.session_summaries;
  });
};

// Selector hook for getting formatted loading time (e.g., "2.5s ago")
export const useHerdModulesLoadingTimeAgo = () => {
  return useSelector((state: any) => {
    const loadingDuration =
      state.scout.loading_performance.herd_modules_loaded_in_ms;
    if (!loadingDuration) return null;

    // Since we store the duration, we need to calculate when it was loaded
    // We'll use the lastRefreshed timestamp from the store
    const lastRefreshed = state.scout.lastRefreshed;
    if (!lastRefreshed) return null;

    const now = Date.now();
    const timeSinceLoaded = now - lastRefreshed;

    // Handle edge case where timeSinceLoaded might be negative
    if (timeSinceLoaded < 0) {
      return "just now";
    }

    const diffSeconds = Math.floor(timeSinceLoaded / 1000);
    const diffMinutes = Math.floor(diffSeconds / 60);
    const diffHours = Math.floor(diffMinutes / 60);

    if (diffHours > 0) {
      return `${diffHours}h ago`;
    } else if (diffMinutes > 0) {
      return `${diffMinutes}m ago`;
    } else if (diffSeconds > 0) {
      return `${diffSeconds}s ago`;
    } else {
      return "just now";
    }
  });
};
