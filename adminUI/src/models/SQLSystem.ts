import { SystemType } from "./api";
import { MarketTrade } from "./Market";
import { SQLWaypoint } from "./SQLWaypoint";

export interface SystemResp {
  system: SQLSystem;
  waypoints: { waypoint: SQLWaypoint; trade_goods: MarketTrade[] }[];
}

export interface SQLSystem {
  sector_symbol: string;
  symbol: string;
  system_type: SystemType;
  x: number;
  y: number;
  waypoints: number | undefined;
}
