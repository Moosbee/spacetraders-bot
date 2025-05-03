import { createSelector, type PayloadAction } from "@reduxjs/toolkit";
import RustShip from "../../models/ship";
import { createAppSlice } from "../createAppSlice";

export interface ConfigSliceState {
  ships: Record<string, RustShip>;
}

const initialState: ConfigSliceState = {
  ships: {},
};

// If you are not using async thunks you can use the standalone `createSlice`.
export const shipSlice = createAppSlice({
  name: "ships",
  // `createSlice` will infer the state type from the `initialState` argument
  initialState,
  // The `reducers` field lets us define reducers and generate associated actions
  reducers: (create) => ({
    // Use the `PayloadAction` type to declare the contents of `action.payload`
    setShips: create.reducer(
      (state, action: PayloadAction<Record<string, RustShip>>) => {
        // Redux Toolkit allows us to write "mutating" logic in reducers. It
        // doesn't actually mutate the state because it uses the Immer library,
        // which detects changes to a "draft state" and produces a brand new
        // immutable state based off those changes

        state.ships = action.payload;
      }
    ),

    setShip: create.reducer((state, action: PayloadAction<RustShip>) => {
      state.ships[action.payload.symbol] = action.payload;
    }),

    resetShips: create.reducer((state) => {
      state.ships = {};
    }),
  }),
  // You can define your selectors here. These selectors receive the slice
  // state as their first argument.
  selectors: {
    selectAllShipsMap: (state) => state.ships,
    selectAllShipsArray: createSelector(
      (state) => state.ships,
      (ships) => Object.values<RustShip>(ships)
    ),
    selectShip: (state, symbol?: string) =>
      symbol ? state.ships[symbol] : undefined,
  },
});

// Action creators are generated for each case reducer function.
export const { setShip, setShips, resetShips } = shipSlice.actions;

// Selectors returned by `slice.selectors` take the root state as their first argument.
export const { selectAllShipsMap, selectShip, selectAllShipsArray } =
  shipSlice.selectors;
