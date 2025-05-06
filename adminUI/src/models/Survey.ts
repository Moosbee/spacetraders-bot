import { SurveySizeEnum, TradeSymbol } from "./api";

export interface Survey {
  ship_info_before: number;
  ship_info_after: number;
  signature: string;
  waypoint_symbol: string;
  deposits: TradeSymbol[];
  expiration: string;
  size: SurveySizeEnum;
  exhausted_since?: string;
  created_at: string;
  updated_at: string;
}
