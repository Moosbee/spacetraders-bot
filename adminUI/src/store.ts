import { shared } from "use-broadcast-ts";
import { create } from "zustand";
import { createJSONStorage, devtools } from "zustand/middleware";
import { DbAgent } from "./models/Agent";
import RustShip from "./models/ship";
import { SQLSystem } from "./models/SQLSystem";
import { SQLWaypoint } from "./models/SQLWaypoint";
// import type {} from '@redux-devtools/extension' // required for devtools typing

// // Custom storage object
// export const indexDBStorage: StateStorage = {
//   getItem: async (name: string): Promise<string | null> => {
//     console.log(name, "has been retrieved");
//     return (await getIDB(name)) || null;
//   },
//   setItem: async (name: string, value: string): Promise<void> => {
//     console.log(name, "with value", value, "has been saved");
//     await setIDB(name, value);
//   },
//   removeItem: async (name: string): Promise<void> => {
//     console.log(name, "has been deleted");
//     await delIDB(name);
//   },
// };

// const indexDBJsonStorage = createJSONStorage(() => indexDBStorage);

const storage = createJSONStorage(() => localStorage);

type State = {
  darkMode: boolean;
  ships: Record<string, RustShip>;
  sliderCollapsed: boolean;
  myAgent: DbAgent;
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
  setAgent: (agent: DbAgent) => void;
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
    ship_count: 0,
    starting_faction: "",
    symbol: "",
    created_at: "",
  },
  selectedShipSymbol: undefined,
  selectedWaypointSymbol: undefined,
  selectedSystemSymbol: undefined,
  systems: {},
  websocketConnected: false,
};

const useMyStore = create<RootState>()(
  devtools(
    // persist(
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
    // ),
    // {
    //   name: "root-state",
    // }
  )
);

export default useMyStore;

export const backendUrl = "127.0.0.1:8780";

// // Database name and version
// const DB_NAME = "keyValueStore";
// const DB_VERSION = 1;
// const STORE_NAME = "keyValuePairs";

// /**
//  * Opens the IndexedDB database connection
//  * @returns A promise that resolves to an IDBDatabase instance
//  */
// function openDatabase(): Promise<IDBDatabase> {
//   return new Promise((resolve, reject) => {
//     const request = indexedDB.open(DB_NAME, DB_VERSION);

//     request.onerror = (event) => {
//       reject(`Database error: ${(event.target as IDBOpenDBRequest).error}`);
//     };

//     request.onsuccess = (event) => {
//       resolve((event.target as IDBOpenDBRequest).result);
//     };

//     request.onupgradeneeded = (event) => {
//       const db = (event.target as IDBOpenDBRequest).result;
//       // Create the object store if it doesn't exist
//       if (!db.objectStoreNames.contains(STORE_NAME)) {
//         db.createObjectStore(STORE_NAME);
//       }
//     };
//   });
// }

// /**
//  * Gets a value from IndexedDB by key
//  * @param name The key to retrieve
//  * @returns A promise that resolves to the stored string value, or null if not found
//  */
// function getIDB(name: string): Promise<string | null> {
//   return new Promise((resolve, reject) => {
//     openDatabase()
//       .then((db) => {
//         const transaction = db.transaction([STORE_NAME], "readonly");
//         const store = transaction.objectStore(STORE_NAME);
//         const request = store.get(name);

//         request.onerror = (event) => {
//           db.close();
//           reject(
//             `Error retrieving value: ${(event.target as IDBRequest).error}`
//           );
//         };

//         request.onsuccess = (event) => {
//           db.close();
//           resolve((event.target as IDBRequest).result || null);
//         };
//       })
//       .catch((error) => {
//         reject(`Database connection error: ${error}`);
//       });
//   });
// }

// /**
//  * Stores a value in IndexedDB with the given key
//  * @param name The key to store
//  * @param value The string value to store
//  * @returns A promise that resolves when the value is stored
//  */
// function setIDB(name: string, value: string): Promise<void> {
//   return new Promise((resolve, reject) => {
//     openDatabase()
//       .then((db) => {
//         const transaction = db.transaction([STORE_NAME], "readwrite");
//         const store = transaction.objectStore(STORE_NAME);
//         const request = store.put(value, name);

//         request.onerror = (event) => {
//           db.close();
//           reject(`Error storing value: ${(event.target as IDBRequest).error}`);
//         };

//         transaction.oncomplete = () => {
//           db.close();
//           resolve();
//         };
//       })
//       .catch((error) => {
//         reject(`Database connection error: ${error}`);
//       });
//   });
// }

// /**
//  * Deletes a value from IndexedDB by key
//  * @param name The key to delete
//  * @returns A promise that resolves when the value is deleted
//  */
// function delIDB(name: string): Promise<void> {
//   return new Promise((resolve, reject) => {
//     openDatabase()
//       .then((db) => {
//         const transaction = db.transaction([STORE_NAME], "readwrite");
//         const store = transaction.objectStore(STORE_NAME);
//         const request = store.delete(name);

//         request.onerror = (event) => {
//           db.close();
//           reject(`Error deleting value: ${(event.target as IDBRequest).error}`);
//         };

//         transaction.oncomplete = () => {
//           db.close();
//           resolve();
//         };
//       })
//       .catch((error) => {
//         reject(`Database connection error: ${error}`);
//       });
//   });
// }
