import { shared } from "use-broadcast-ts";
import { create } from "zustand";
import { createJSONStorage, devtools, persist } from "zustand/middleware";
import { Agent } from "./models/api";
import RustShip from "./models/ship";
import { SQLSystem } from "./models/SQLSystem";
import { SQLWaypoint } from "./models/SQLWaypoint";
// import type {} from '@redux-devtools/extension' // required for devtools typing

const storage = createJSONStorage(() => localStorage);

// // Custom storage object
// const storage: StateStorage = {
//   getItem: async (key: string) => {
//     const db = await indexedDB.open("myDB", 1);
//     const tx = db.transaction("myStore", "readonly");
//     const store = tx.objectStore("myStore");
//     const request = store.get(key);
//     const result = await new Promise((resolve) => {
//       request.onsuccess = () => resolve(request.result);
//     });
//     return result;
//   },
//   setItem: async (key: string, value: any) => {
//     const db = await indexedDB.open("myDB", 1);
//     const tx = db.transaction("myStore", "readwrite");
//     const store = tx.objectStore("myStore");
//     store.put(value, key);
//     await new Promise((resolve) => {
//       tx.oncomplete = resolve;
//     });
//   },
//   removeItem: async (key: string) => {
//     const db = await indexedDB.open("myDB", 1);
//     const tx = db.transaction("myStore", "readwrite");
//     const store = tx.objectStore("myStore");
//     store.delete(key);
//     await new Promise((resolve) => {
//       tx.oncomplete = resolve;
//     });
//   },
// };

type State = {
  darkMode: boolean;
  ships: Record<string, RustShip>;
  sliderCollapsed: boolean;
  myAgent: Agent;
  selectedShipSymbol: string | undefined;
  selectedWaypointSymbol:
    | { systemSymbol: string; waypointSymbol: string }
    | undefined;
  selectedSystemSymbol: string | undefined;
  systems: Record<
    string,
    {
      system: SQLSystem;
      waypoints: SQLWaypoint[];
    }
  >;
  websocketConnected: boolean;
};

type Actions = {
  setDarkMode: (darkMode: boolean) => void;
  setShips: (ships: Record<string, RustShip>) => void;
  setShip: (ship: RustShip) => void;
  setSliderCollapsed: (collapsed: boolean) => void;
  setSelectedShipSymbol: (symbol: string | undefined) => void;
  setSelectedWaypointSymbol: (
    waypoint:
      | {
          systemSymbol: string;
          waypointSymbol: string;
        }
      | undefined
  ) => void;
  setSelectedSystemSymbol: (systemSymbol: string | undefined) => void;
  setSystems: (
    waypoints: Record<
      string,
      {
        system: SQLSystem;
        waypoints: SQLWaypoint[];
      }
    >
  ) => void;
  setSystem: (system: SQLSystem, waypoints: SQLWaypoint[]) => void;
  setWebsocketConnected: (websocketConnected: boolean) => void;
  setAgent: (agent: Agent) => void;
  reset: () => void;
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
  selectedShipSymbol: undefined,
  selectedWaypointSymbol: undefined,
  selectedSystemSymbol: undefined,
  systems: {},
  websocketConnected: false,
};

const useMyStore = create<RootState>()(
  devtools(
    persist(
      shared((set) => ({
        ...initialState,
        reset: () => set(initialState),
        setDarkMode: (darkMode) => set({ darkMode }),
        setShips: (ships) => set({ ships: ships }),
        setShip: (ship) =>
          set((state) => ({ ships: { ...state.ships, [ship.symbol]: ship } })),
        setSliderCollapsed: (collapsed) => set({ sliderCollapsed: collapsed }),
        setSelectedShipSymbol: (symbol) => set({ selectedShipSymbol: symbol }),
        setSelectedWaypointSymbol: (waypoint) =>
          set({
            selectedWaypointSymbol: waypoint,
          }),
        setSelectedSystemSymbol: (systemSymbol) =>
          set({ selectedSystemSymbol: systemSymbol }),
        setSystems: (systems) => set({ systems: systems }),
        setSystem: (system, waypoints) =>
          set((state) => ({
            systems: {
              ...state.systems,
              [system.symbol]: { system: system, waypoints: waypoints },
            },
          })),
        setWebsocketConnected: (websocketConnected) =>
          set({ websocketConnected }),
        setAgent: (agent) => set({ myAgent: agent }),
      })),
      {
        name: "root-channel",
        storage: storage,
      }
    ),
    {
      name: "root-state",
    }
  )
);

export default useMyStore;

export const backendUrl = "127.0.0.1:8780";
