import { createSlice } from "@reduxjs/toolkit";
import { IEventWithTags, IUser } from "../types/db";
import { IHerdModule, EnumHerdModulesLoadingState } from "../types/herd_module";
import { EnumDataSource, IDataSourceInfo } from "../types/data_source";
import { MapDeviceIdToConnectivity } from "../types/connectivity";

export enum EnumScoutStateStatus {
  LOADING = "LOADING",
  DONE_LOADING = "DONE_LOADING",
}

export interface LoadingPerformance {
  herd_modules_loaded_in_ms: number | null;
  // Detailed timing measurements for each portion of the loading process
  herd_modules_api_server_processing_ms: number | null;
  herd_modules_api_total_request_ms: number | null;
  user_api_duration_ms: number | null;
  data_processing_duration_ms: number | null;
  cache_load_duration_ms: number | null;
}

export interface ScoutState {
  herd_modules: IHerdModule[];
  status: EnumScoutStateStatus;
  herd_modules_loading_state: EnumHerdModulesLoadingState;
  loading_performance: LoadingPerformance;
  active_herd_id: string | null;
  active_device_id: string | null;
  lastRefreshed: number;
  user: IUser | null;
  // Data source tracking
  data_source: EnumDataSource;
  data_source_info: IDataSourceInfo | null;
  active_herd_gps_trackers_connectivity: MapDeviceIdToConnectivity;
}

// Root state type for the entire application
export interface RootState {
  scout: ScoutState;
}

const initialState: ScoutState = {
  herd_modules: [],
  status: EnumScoutStateStatus.LOADING,
  herd_modules_loading_state: EnumHerdModulesLoadingState.NOT_LOADING,
  loading_performance: {
    herd_modules_loaded_in_ms: null,
    herd_modules_api_server_processing_ms: null,
    herd_modules_api_total_request_ms: null,
    user_api_duration_ms: null,
    data_processing_duration_ms: null,
    cache_load_duration_ms: null,
  },
  lastRefreshed: 0,
  active_herd_id: null,
  active_device_id: null,
  user: null,
  // Initialize data source tracking
  data_source: EnumDataSource.UNKNOWN,
  data_source_info: null,
  active_herd_gps_trackers_connectivity: {},
};

export const scoutSlice = createSlice({
  name: "scout",
  initialState,
  reducers: {
    setHerdModules: (state, action) => {
      state.herd_modules = action.payload;
      state.lastRefreshed = Date.now();
    },
    setStatus: (state, action) => {
      state.status = action.payload;
    },
    setHerdModulesLoadingState: (state, action) => {
      state.herd_modules_loading_state = action.payload;
    },
    setLoadingPerformance: (state, action) => {
      state.loading_performance = {
        ...state.loading_performance,
        ...action.payload,
      };
    },
    setHerdModulesLoadedInMs: (state, action) => {
      state.loading_performance.herd_modules_loaded_in_ms = action.payload;
    },
    setHerdModulesApiServerProcessingDuration: (state, action) => {
      state.loading_performance.herd_modules_api_server_processing_ms =
        action.payload;
    },
    setHerdModulesApiTotalRequestDuration: (state, action) => {
      state.loading_performance.herd_modules_api_total_request_ms =
        action.payload;
    },
    setUserApiDuration: (state, action) => {
      state.loading_performance.user_api_duration_ms = action.payload;
    },
    setDataProcessingDuration: (state, action) => {
      state.loading_performance.data_processing_duration_ms = action.payload;
    },
    setCacheLoadDuration: (state, action) => {
      state.loading_performance.cache_load_duration_ms = action.payload;
    },
    setActiveHerdId: (state, action) => {
      state.active_herd_id = action.payload;
      state.active_herd_gps_trackers_connectivity = {};
    },
    setActiveDeviceId: (state, action) => {
      state.active_device_id = action.payload;
    },
    setDataSource: (state, action) => {
      state.data_source = action.payload;
    },
    setDataSourceInfo: (state, action) => {
      state.data_source_info = action.payload;
    },
    updateSessionSummariesForHerdModule: (state, action) => {
      const { herd_id, session_summaries } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id,
      );
      if (herd_module) {
        herd_module.session_summaries = session_summaries;
      }
    },

    appendPlansToHerdModule: (state, action) => {
      const { herd_id, plan } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id,
      );
      if (herd_module) {
        herd_module.plans = [...herd_module.plans, plan];
      }
    },
    updateDeviceForHerdModule: (state, action) => {
      const { herd_id, device } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id,
      );
      if (herd_module) {
        herd_module.devices = herd_module.devices.map((d) =>
          d.id === device.id ? device : d,
        );
      }
    },
    // append device to herd module
    addNewDeviceToHerdModule: (state, action) => {
      const { herd_id, device } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id,
      );
      if (herd_module) {
        herd_module.devices = [...herd_module.devices, device];
      }
    },
    // NOTE: Tag management will need to be updated to work with RTK Query
    // These actions are commented out until we implement tag management with RTK Query
    // addTag(state, action) { ... },
    // deleteTag(state, action) { ... },
    // updateTag(state, action) { ... },
    addDevice(state, action) {
      for (const herd_module of state.herd_modules) {
        if (herd_module.herd.id === action.payload.herd_id) {
          herd_module.devices.push(action.payload);
          return;
        }
      }
    },
    deleteDevice(state, action) {
      for (const herd_module of state.herd_modules) {
        herd_module.devices = herd_module.devices.filter(
          (device) => device.id !== action.payload.id,
        );
      }
    },
    updateDevice(state, action) {
      for (const herd_module of state.herd_modules) {
        for (const device of herd_module.devices) {
          if (device.id === action.payload.id) {
            device.name = action.payload.name;
            device.id = action.payload.id;
            device.device_type = action.payload.device_type;
            device.altitude = action.payload.altitude;
            device.latitude = action.payload.latitude;
            device.longitude = action.payload.longitude;
            device.domain_name = action.payload.domain_name;
            return;
          }
        }
      }
    },
    addPlan(state, action) {
      for (const herd_module of state.herd_modules) {
        if (herd_module.herd.id === action.payload.herd_id) {
          herd_module.plans.push(action.payload);
          return;
        }
      }
    },
    deletePlan(state, action) {
      for (const herd_module of state.herd_modules) {
        herd_module.plans = herd_module.plans.filter(
          (plan) => plan.id !== action.payload.id,
        );
      }
    },
    updatePlan(state, action) {
      for (const herd_module of state.herd_modules) {
        for (const plan of herd_module.plans) {
          if (plan.id === action.payload.id) {
            plan.name = action.payload.name;
            plan.instructions = action.payload.instructions;
            plan.herd_id = action.payload.herd_id;
            return;
          }
        }
      }
    },
    // NOTE: Events, sessions, and artifacts are now handled by RTK Query
    // These actions have been removed as they're no longer needed
    setUser: (state, action) => {
      state.user = action.payload;
    },
    setActiveHerdGpsTrackersConnectivity: (state, action) => {
      state.active_herd_gps_trackers_connectivity = action.payload;
    },
  },
});

// Action creators are generated for each case reducer function
export const {
  setHerdModules,
  setStatus,
  setHerdModulesLoadingState,
  setLoadingPerformance,
  setHerdModulesLoadedInMs,
  setHerdModulesApiServerProcessingDuration,
  setHerdModulesApiTotalRequestDuration,
  setUserApiDuration,
  setDataProcessingDuration,
  setCacheLoadDuration,
  setActiveHerdId,
  setActiveDeviceId,
  setDataSource,
  setDataSourceInfo,
  updateSessionSummariesForHerdModule,
  appendPlansToHerdModule,
  setUser,
  addNewDeviceToHerdModule,
  updateDeviceForHerdModule,
  addDevice,
  deleteDevice,
  updateDevice,
  addPlan,
  deletePlan,
  updatePlan,
  setActiveHerdGpsTrackersConnectivity,
} = scoutSlice.actions;

export default scoutSlice.reducer;
