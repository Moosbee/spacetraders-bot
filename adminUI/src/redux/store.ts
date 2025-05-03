import createIdbStorage from "@piotr-cz/redux-persist-idb-storage";
import {
  Action,
  combineSlices,
  configureStore,
  ThunkAction,
} from "@reduxjs/toolkit";
import { setupListeners } from "@reduxjs/toolkit/query";
import type { PersistConfig } from "redux-persist";
import { persistReducer, persistStore } from "redux-persist";
import autoMergeLevel2 from "redux-persist/lib/stateReconciler/autoMergeLevel2";
import {
  createStateSyncMiddleware,
  initMessageListener,
} from "redux-state-sync";
import { Prettify } from "../utils/utils";
import { agentSlice } from "./slices/agentSlice";
import { configSlice } from "./slices/configSlice";
import { mapSlice } from "./slices/mapSlice";
import { shipSlice } from "./slices/shipSlice";
import { systemSlice } from "./slices/systemSlice";

// Create a persist config for Redux Persist
const persistConfig: PersistConfig<RootState> = {
  key: "root", // Key for the persisted data
  stateReconciler: autoMergeLevel2, // see "Merge Process" section for details.
  // blacklist: [],
  storage: createIdbStorage({ name: "myApp", storeName: "keyval" }),
  serialize: false, // Data serialization is not required and disabling it allows you to inspect storage value in DevTools; Available since redux-persist@5.4.0
  // @ts-expect-error idk
  deserialize: false, // Required to bear same value as `serialize` since redux-persist@6.0
};

// `combineSlices` automatically combines the reducers using
// their `reducerPath`s, therefore we no longer need to call `combineReducers`.
//const rootReducer = combineSlices(counterSlice, quotesApiSlice, surveySlice);
// because persist-redux we need to call `combineReducers`
const rootReducer = combineSlices(
  configSlice,
  mapSlice,
  shipSlice,
  systemSlice,
  agentSlice
);

// Wrap the rootReducer with persistReducer
const persistedReducer = persistReducer(persistConfig, rootReducer);

// Infer the `RootState` type from the root reducer
export type RootState = ReturnType<typeof rootReducer>;

// The store setup is wrapped in `makeStore` to allow reuse
// when setting up tests that need the same store config
// eslint-disable-next-line @typescript-eslint/no-unused-vars
export const makeStore = (_preloadedState?: Partial<RootState>) => {
  const store = configureStore({
    reducer: persistedReducer,
    devTools: true,
    // Adding the api middleware enables caching, invalidation, polling,
    // and other useful features of `rtk-query`.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    middleware: (getDefaultMiddleware: any) => {
      const data1 = getDefaultMiddleware({
        serializableCheck: {
          // Ignore these action types
          ignoredActions: ["persist/PERSIST", "persist/REHYDRATE"],
          ignoredPaths: [],
        },

        immutableCheck: {
          // Ignore state paths, e.g. state for 'items':
          ignoredPaths: [],
        },
      });
      const data2 = data1.concat(
        createStateSyncMiddleware({
          blacklist: ["persist/PERSIST", "persist/REHYDRATE"],
          channel: "reduxStateSync",
        })
      );
      return data2;
    },
  });
  // configure listeners using the provided defaults
  // optional, but required for `refetchOnFocus`/`refetchOnReconnect` behaviors
  setupListeners(store.dispatch);
  const tStore = store as Prettify<typeof store>;
  return tStore;
};

export const store = makeStore();
initMessageListener(store);
export const persistor = persistStore(store);

// Infer the type of `store`
export type AppStore = Prettify<typeof store>;
// Infer the `AppDispatch` type from the store itself
export type AppDispatch = AppStore["dispatch"];
export type AppThunk<ThunkReturnType = void> = ThunkAction<
  ThunkReturnType,
  RootState,
  unknown,
  Action
>;
