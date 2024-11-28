import { shared } from "use-broadcast-ts";
import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";
import RustShip from "./models/ship";
// import type {} from '@redux-devtools/extension' // required for devtools typing

interface RootState {
  ships: Record<string, RustShip>;
  setShips: (ships: Record<string, RustShip>) => void;
  setShip: (ship: RustShip) => void;
}

const useStore = create<RootState>()(
  devtools(
    persist(
      shared((set) => ({
        ships: {},
        setShips: (ships) => set({ ships: ships }),
        setShip: (ship) =>
          set((state) => ({ ships: { ...state.ships, [ship.symbol]: ship } })),
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

export default useStore;
