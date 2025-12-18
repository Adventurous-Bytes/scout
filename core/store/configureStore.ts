import { configureStore } from "@reduxjs/toolkit";
import { setupListeners } from "@reduxjs/toolkit/query";
import scoutReducer from "./scout";
import { scoutApi } from "./api";

// Configure the Redux store
export const configureScoutStore = () => {
  const store = configureStore({
    reducer: {
      scout: scoutReducer,
      [scoutApi.reducerPath]: scoutApi.reducer,
    },
    middleware: (getDefaultMiddleware) =>
      getDefaultMiddleware({
        serializableCheck: {
          // Ignore these action types for serializable check
          ignoredActions: [
            "persist/PERSIST",
            "persist/REHYDRATE",
            "persist/REGISTER",
            // RTK Query actions
            "scoutApi/executeQuery/pending",
            "scoutApi/executeQuery/fulfilled",
            "scoutApi/executeQuery/rejected",
            "scoutApi/executeMutation/pending",
            "scoutApi/executeMutation/fulfilled",
            "scoutApi/executeMutation/rejected",
          ],
          // Ignore these field paths in all actions
          ignoredActionsPaths: ["meta.arg", "payload.timestamp"],
          // Ignore these paths in the state
          ignoredPaths: ["scoutApi.queries", "scoutApi.mutations"],
        },
      }).concat(scoutApi.middleware),
    devTools: process.env.NODE_ENV !== "production",
  });

  // Enable listener behavior for the store
  setupListeners(store.dispatch);

  return store;
};

export type RootState = ReturnType<
  ReturnType<typeof configureScoutStore>["getState"]
>;
export type AppDispatch = ReturnType<typeof configureScoutStore>["dispatch"];

// Create a default store instance
export const store = configureScoutStore();
