import {
  ShipEngineSymbolEnum,
  ShipFrameSymbolEnum,
  ShipModuleSymbolEnum,
  ShipMountSymbolEnum,
  ShipReactorSymbolEnum,
  ShipType,
  SupplyLevel,
} from "./api";

export interface ShipTransaction {
  agent_symbol: string;
  price: number;
  ship_type: ShipType;
  timestamp: string;
  waypoint_symbol: string;
}

export interface ShipyardShipType {
  created_at: string;
  id: number;
  ship_type: ShipType;
  shipyard_id: number;
}

export interface ShipyardShip {
  activity: string;
  created_at: string;
  crew_capacity: number;
  crew_requirement: number;
  engine_quality: number;
  engine_type: ShipEngineSymbolEnum;
  frame_quality: number;
  frame_type: ShipFrameSymbolEnum;
  id: number;
  modules: ShipModuleSymbolEnum[];
  mounts: ShipMountSymbolEnum[];
  name: string;
  purchase_price: number;
  reactor_quality: number;
  reactor_type: ShipReactorSymbolEnum;
  ship_type: ShipType;
  supply: SupplyLevel;
  waypoint_symbol: string;
}

export interface Shipyard {
  created_at: string;
  id: number;
  modifications_fee: number;
  waypoint_symbol: string;
}
