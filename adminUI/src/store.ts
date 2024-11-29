import { shared } from "use-broadcast-ts";
import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";
import { Agent } from "./models/api";
import RustShip from "./models/ship";
// import type {} from '@redux-devtools/extension' // required for devtools typing

type State = {
  darkMode: boolean;
  ships: Record<string, RustShip>;
  sliderCollapsed: boolean;
  myAgent: Agent;
  selectSelectedShipSymbol: string | undefined;
  selectSelectedWaypointSymbol:
    | { systemSymbol: string; waypointSymbol: string }
    | undefined;
  selectSelectedSystemSymbol: string | undefined;
};

type Actions = {
  setDarkMode: (darkMode: boolean) => void;
  setShips: (ships: Record<string, RustShip>) => void;
  setShip: (ship: RustShip) => void;
  setSliderCollapsed: (collapsed: boolean) => void;
};

export type RootState = State & Actions;

const initialState: State = {
  darkMode: true,
  ships: {},
  sliderCollapsed: false,
  myAgent: {
    credits: 0,
    headquarters: "",
    shipCount: 0,
    startingFaction: "",
    symbol: "",
  },
  selectSelectedShipSymbol: "",
  selectSelectedWaypointSymbol: { systemSymbol: "", waypointSymbol: "" },
  selectSelectedSystemSymbol: "",
};

const useMyStore = create<RootState>()(
  devtools(
    persist(
      shared((set) => ({
        ...initialState,
        setDarkMode: (darkMode) => set({ darkMode }),
        setShips: (ships) => set({ ships: ships }),
        setShip: (ship) =>
          set((state) => ({ ships: { ...state.ships, [ship.symbol]: ship } })),
        setSliderCollapsed: (collapsed) => set({ sliderCollapsed: collapsed }),
      })),
      {
        name: "root-channel",
      }
    ),
    {
      name: "root-state",
    }
  )
);

export default useMyStore;
