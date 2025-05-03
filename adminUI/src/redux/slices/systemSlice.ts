import type { PayloadAction } from "@reduxjs/toolkit";
import { SQLSystem } from "../../models/SQLSystem";
import { SQLWaypoint } from "../../models/SQLWaypoint";
import { createAppSlice } from "../createAppSlice";

export interface ConfigSliceState {
  systems: Record<
    string,
    {
      system: SQLSystem;
      waypoints: SQLWaypoint[];
    }
  >;
}

const initialState: ConfigSliceState = {
  systems: {},
};

// If you are not using async thunks you can use the standalone `createSlice`.
export const systemSlice = createAppSlice({
  name: "systems",
  // `createSlice` will infer the state type from the `initialState` argument
  initialState,
  // The `reducers` field lets us define reducers and generate associated actions
  reducers: (create) => ({
    // Use the `PayloadAction` type to declare the contents of `action.payload`
    setSystem: create.reducer(
      (
        state,
        action: PayloadAction<{
          system: SQLSystem;
          waypoints: SQLWaypoint[];
        }>
      ) => {
        // Redux Toolkit allows us to write "mutating" logic in reducers. It
        // doesn't actually mutate the state because it uses the Immer library,
        // which detects changes to a "draft state" and produces a brand new
        // immutable state based off those changes

        state.systems[action.payload.system.symbol] = action.payload;
      }
    ),
    resetSystems: create.reducer((state) => {
      state.systems = {};
    }),
  }),
  // You can define your selectors here. These selectors receive the slice
  // state as their first argument.
  selectors: {
    selectAllSystems: (state) => state.systems,
    selectSystem: (state, symbol?: string) =>
      symbol ? state.systems[symbol] : undefined,
  },
});

// Action creators are generated for each case reducer function.
export const { setSystem, resetSystems } = systemSlice.actions;

// Selectors returned by `slice.selectors` take the root state as their first argument.
export const { selectAllSystems, selectSystem } = systemSlice.selectors;
