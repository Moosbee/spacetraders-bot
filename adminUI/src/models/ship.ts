import { ShipNavFlightMode, ShipNavStatus, ShipRole, TradeSymbol } from "./api";

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
  inventory: [TradeSymbol, number][];
}

interface Fuel {
  capacity: number;
  current: number;
}

interface RustShip {
  role: SystemShipRole;
  registration_role: ShipRole;
  symbol: string;
  engine_speed: number;
  cooldown_expiration: string | null;
  nav: Navigation;
  cargo: Cargo;
  fuel: Fuel;
}

export type SystemShipRole =
  (typeof SystemShipRole)[keyof typeof SystemShipRole];
export const SystemShipRole = {
  Construction: "Construction",
  Trader: "Trader",
  Contract: "Contract",
  Scraper: "Scraper",
  Mining: "Mining",
  Manuel: "Manuel",
} as const;

export default RustShip;
