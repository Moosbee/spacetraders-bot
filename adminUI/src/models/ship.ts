import {
  ShipModuleSymbolEnum,
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
      distance: number;
      refuel_to: number;
      fuel_in_cargo: number;
    }[];
    connections: {
      start: unknown;
      end: unknown;
      flight_mode: string;
      distance: number;
      fuel_cost: number;
      travel_time: number;
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

interface Modules {
  modules: ShipModuleSymbolEnum[];
}

export interface Condition {
  integrity: number;
  condition: number;
}

interface RustShip {
  role: SystemShipRoles;
  status: SystemShipRole;
  registration_role: ShipRole;
  symbol: string;
  display_name: string;
  active: boolean;
  engine_speed: number;
  nav: Navigation;
  cargo: Cargo;
  fuel: Fuel;
  modules: Modules;
  mounts: Mounts;
  cooldown_expiration: string | null;

  conditions: {
    engine: Condition;
    frame: Condition;
    reactor: Condition;
  };
}

export interface ShipInfo {
  symbol: string;
  displayName: string;
  role: SystemShipRoles;
  active: boolean;
}

export type SystemShipRoles =
  (typeof SystemShipRoles)[keyof typeof SystemShipRoles];
export const SystemShipRoles = {
  Construction: "Construction",
  Trader: "Trader",
  TempTrader: "TempTrader",
  Contract: "Contract",
  Scraper: "Scraper",
  Mining: "Mining",
  Charter: "Charter",
  Manuel: "Manuel",
} as const;

export type SystemShipRole =
  | { type: "Construction"; data: ConstructionData }
  | { type: "Trader"; data: TraderData }
  | { type: "Contract"; data: ContractData }
  | { type: "Scraper"; data: null }
  | { type: "Mining"; data: MiningData }
  | { type: "Manuel"; data: null } // Default role
  | { type: "Charting"; data: ChartingData };

interface ConstructionData {
  cycle?: number;
  shipment_id?: number;
  shipping_status?: ShippingStatus;
  waiting_for_manager: boolean;
}

interface TraderData {
  shipment_id?: number;
  cycle?: number;
  shipping_status?: ShippingStatus;
  waiting_for_manager: boolean;
}

interface ChartingData {
  cycle?: number;
  // shipping_status?: ShippingStatus;
  waiting_for_manager: boolean;
  waypoint_symbol?: string;
}

interface ContractData {
  contract_id?: string;
  run_id?: number;
  cycle?: number;
  shipping_status?: ShippingStatus;
  waiting_for_manager: boolean;
}

interface MiningData {
  assignment: MiningShipAssignment;
}

enum ShippingStatus {
  InTransitToPurchase = "InTransitToPurchase",
  Purchasing = "Purchasing",
  InTransitToDelivery = "InTransitToDelivery",
  Delivering = "Delivering",
  Unknown = "Unknown",
}

type MiningShipAssignment =
  | {
      type: "Transporter";
      data: {
        state: TransporterState;
        waypoint_symbol?: string;
        cycles?: number;
      };
    }
  | {
      type: "Extractor";
      data: {
        state: ExtractorState;
        waypoint_symbol?: string;
        extractions?: number;
      };
    }
  | {
      type: "Siphoner";
      data: {
        state: SiphonerState;
        waypoint_symbol?: string;
        extractions?: number;
      };
    }
  | { type: "Surveyor" }
  | { type: "Idle" }
  | { type: "Useless" };

enum TransporterState {
  InTransitToAsteroid = "InTransitToAsteroid",
  LoadingCargo = "LoadingCargo",
  WaitingForCargo = "WaitingForCargo",
  InTransitToMarket = "InTransitToMarket",
  SellingCargo = "SellingCargo",
  Unknown = "Unknown",
}

type SiphonerState = ExtractorState;

enum ExtractorState {
  InTransit = "InTransit",
  Mining = "Mining",
  OnCooldown = "OnCooldown",
  InvFull = "InvFull",
  Unknown = "Unknown",
}

export default RustShip;
