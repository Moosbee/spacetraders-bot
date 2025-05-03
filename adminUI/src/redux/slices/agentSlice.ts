import type { PayloadAction } from "@reduxjs/toolkit";
import { DbAgent } from "../../models/Agent";
import { createAppSlice } from "../createAppSlice";

export interface ConfigSliceState {
  myAgent: DbAgent;
}

const initialState: ConfigSliceState = {
  myAgent: {
    credits: -1,
    headquarters: "",
    ship_count: -1,
    starting_faction: "",
    symbol: "",
    created_at: "",
  },
};

// If you are not using async thunks you can use the standalone `createSlice`.
export const agentSlice = createAppSlice({
  name: "agent",
  // `createSlice` will infer the state type from the `initialState` argument
  initialState,
  // The `reducers` field lets us define reducers and generate associated actions
  reducers: (create) => ({
    // Use the `PayloadAction` type to declare the contents of `action.payload`
    setMyAgent: create.reducer((state, action: PayloadAction<DbAgent>) => {
      // Redux Toolkit allows us to write "mutating" logic in reducers. It
      // doesn't actually mutate the state because it uses the Immer library,
      // which detects changes to a "draft state" and produces a brand new
      // immutable state based off those changes

      state.myAgent = action.payload;
    }),
  }),
  // You can define your selectors here. These selectors receive the slice
  // state as their first argument.
  selectors: {
    selectMyAgent: (state) => state.myAgent,
  },
});

// Action creators are generated for each case reducer function.
export const { setMyAgent } = agentSlice.actions;

// Selectors returned by `slice.selectors` take the root state as their first argument.
export const { selectMyAgent } = agentSlice.selectors;
