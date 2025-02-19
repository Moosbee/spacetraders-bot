import {
  ShipMountSymbolEnum,
  ShipNavFlightMode,
  ShipNavStatus,
  ShipRole,
  TradeSymbol,
} from "./api";

interface Navigation {
  flight_mode: ShipNavFlightMode;
  status: ShipNavStatus;
  system_symbol: string;
  waypoint_symbol: string;
  route: {
    // arrival and departure_time are in ISO 8601 format
    arrival: string;
    departure_time: string;
    destination_symbol: string;
    destination_system_symbol: string;
    origin_symbol: string;
    origin_system_symbol: string;
  };
  auto_pilot: {
    // arrival and departure_time are in ISO 8601 format
    arrival: string;
    departure_time: string;
    destination_symbol: string;
    destination_system_symbol: string;
    origin_symbol: string;
    origin_system_symbol: string;
    distance: number;
    fuel_cost: number;
    instructions: {
      start_symbol: string;
      end_symbol: string;
      flight_mode: ShipNavFlightMode;
      start_is_marketplace: boolean;
      refuel_to: number;
      fuel_in_cargo: number;
    }[];
    travel_time: number;
  } | null;
}

interface Cargo {
  capacity: number;
  units: number;
  inventory: Record<TradeSymbol, number>;
}

interface Fuel {
  capacity: number;
  current: number;
}

interface Mounts {
  mounts: ShipMountSymbolEnum[];
}

interface Condition {
  integrity: number;
  condition: number;
}

interface RustShip {
  role: any;
  status: SystemShipRole;
  registration_role: ShipRole;
  symbol: string;
  engine_speed: number;
  cooldown_expiration: string | null;
  nav: Navigation;
  cargo: Cargo;
  fuel: Fuel;
  mounts: Mounts;
  conditions: {
    engine: Condition;
    frame: Condition;
    reactor: Condition;
  };
}

export type SystemShipRoles =
  (typeof SystemShipRoles)[keyof typeof SystemShipRoles];
export const SystemShipRoles = {
  Construction: "Construction",
  Trader: "Trader",
  Contract: "Contract",
  Scraper: "Scraper",
  Mining: "Mining",
  Manuel: "Manuel",
} as const;

export type SystemShipRole =
  | { type: "Construction" }
  | { type: "Trader"; data: [number, number] | null }
  | { type: "Contract"; data: [string, number] | null }
  | { type: "Scraper" }
  | { type: "Mining"; data: string | null }
  | { type: "Manuel" }; // Default role

export default RustShip;
