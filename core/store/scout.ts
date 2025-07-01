import { createSlice } from "@reduxjs/toolkit";
import { IEventWithTags, IUser } from "../types/db";
import { IHerdModule } from "../types/herd_module";

export enum EnumScoutStateStatus {
  LOADING = "LOADING",
  DONE_LOADING = "DONE_LOADING",
}

export interface ScoutState {
  herd_modules: IHerdModule[];
  status: EnumScoutStateStatus;
  active_herd_id: string | null;
  active_device_id: string | null;
  lastRefreshed: number;
  user: IUser | null;
}

const initialState: ScoutState = {
  herd_modules: [],
  status: EnumScoutStateStatus.LOADING,
  lastRefreshed: 0,
  active_herd_id: null,
  active_device_id: null,
  user: null,
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
    setActiveHerdId: (state, action) => {
      state.active_herd_id = action.payload;
    },
    setActiveDeviceId: (state, action) => {
      state.active_device_id = action.payload;
    },
    replaceEventsForHerdModule: (state, action) => {
      const { herd_id, events } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id
      );
      if (herd_module) {
        herd_module.events = events;
      }
    },
    appendEventsToHerdModule: (state, action) => {
      const { herd_id, events } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id
      );
      if (herd_module) {
        herd_module.events = [...herd_module.events, ...events];
      }
    },
    appendPlansToHerdModule: (state, action) => {
      const { herd_id, plan } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id
      );
      if (herd_module) {
        herd_module.plans = [...herd_module.plans, plan];
      }
    },
    updateDeviceForHerdModule: (state, action) => {
      const { herd_id, device } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id
      );
      if (herd_module) {
        herd_module.devices = herd_module.devices.map((d) =>
          d.id === device.id ? device : d
        );
      }
    },
    // append device to herd module
    addNewDeviceToHerdModule: (state, action) => {
      const { herd_id, device } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id
      );
      if (herd_module) {
        herd_module.devices = [...herd_module.devices, device];
      }
    },
    addTag(state, action) {
      for (const herd_module of state.herd_modules) {
        for (const event of herd_module.events) {
          if (event.id === action.payload.event_id && event.tags) {
            event.tags.push(action.payload);
            return;
          }
        }
      }
    },
    deleteTag(state, action) {
      console.log("[Redux] deleteTag action called with:", action.payload);
      console.log("[Redux] deleteTag - Looking for tag ID:", action.payload.id);

      for (const herd_module of state.herd_modules) {
        for (const event of herd_module.events) {
          if (!event.tags) {
            continue;
          }
          console.log(
            `[Redux] deleteTag - Checking event ${event.id}, has ${event.tags.length} tags`
          );
          for (const tag of event.tags) {
            if (tag.id === action.payload.id) {
              console.log(
                `[Redux] deleteTag - Found tag ${tag.id} in event ${event.id}, removing it`
              );
              event.tags = event.tags.filter((t) => t.id !== tag.id);
              console.log(
                `[Redux] deleteTag - After removal, event ${event.id} has ${event.tags.length} tags`
              );
              return;
            }
          }
        }
      }
      console.log("[Redux] deleteTag - Tag not found in any event");
    },
    updateTag(state, action) {
      for (const herd_module of state.herd_modules) {
        for (const event of herd_module.events) {
          if (!event.tags) {
            continue;
          }
          for (const tag of event.tags) {
            if (tag.id === action.payload.id) {
              tag.x = action.payload.x;
              tag.y = action.payload.y;
              tag.width = action.payload.width;
              tag.height = action.payload.height;
              tag.conf = action.payload.conf;
              tag.class_name = action.payload.class_name;
              return;
            }
          }
        }
      }
    },
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
          (device) => device.id !== action.payload.id
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
          (plan) => plan.id !== action.payload.id
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
    // receive a payload with the herd_id and the events that we wish to update the values of
    updateEventValuesForHerdModule: (state, action) => {
      const { herd_id, events } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id
      );
      if (herd_module) {
        herd_module.events = herd_module.events.map((event) => {
          const updated_event = events.find(
            (e: IEventWithTags) => e.id === event.id
          );
          if (updated_event) {
            return updated_event;
          }
          return event;
        });
      }
      state.herd_modules = [...state.herd_modules];
    },
    updatePageIndexForHerdModule: (state, action) => {
      const { herd_id, new_page_index } = action.payload;
      const herd_module = state.herd_modules.find(
        (hm) => hm.herd.id.toString() === herd_id
      );
      if (herd_module) {
        herd_module.events_page_index = new_page_index;
      }
    },
    setUser: (state, action) => {
      state.user = action.payload;
    },
  },
});

// Action creators are generated for each case reducer function
export const {
  setHerdModules,
  setStatus,
  setActiveHerdId,
  setActiveDeviceId,
  appendEventsToHerdModule,
  replaceEventsForHerdModule,
  updateEventValuesForHerdModule,
  updatePageIndexForHerdModule,
  appendPlansToHerdModule,
  setUser,
  addTag,
  deleteTag,
  updateTag,
  addNewDeviceToHerdModule,
  updateDeviceForHerdModule,
  addDevice,
  deleteDevice,
  updateDevice,
  addPlan,
  deletePlan,
  updatePlan,
} = scoutSlice.actions;

export default scoutSlice.reducer;
