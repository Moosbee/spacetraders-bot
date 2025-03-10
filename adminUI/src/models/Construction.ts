import { TradeSymbol } from "./api";

export interface ConstructionMaterial {
  created_at: string;
  fulfilled: number;
  id: number;
  required: number;
  trade_symbol: TradeSymbol;
  updated_at: string;
  waypoint_symbol: string;
}

export interface ConstructionMaterialSummary {
  id: number;
  waypoint_symbol: string;
  trade_symbol: TradeSymbol;
  required: number;
  fulfilled: number;
  created_at: string;
  updated_at: string;
  sum?: number;
  expenses?: number;
  income?: number;
}

export interface ConstructionShipment {
  id: number;
  material_id: number;
  construction_site_waypoint: string;
  ship_symbol: string;
  trade_symbol: TradeSymbol;
  units: number;
  purchase_waypoint: string;
  created_at: string;
  updated_at: string;
  status: string;
  sum?: number;
  expenses?: number;
  income?: number;
}
