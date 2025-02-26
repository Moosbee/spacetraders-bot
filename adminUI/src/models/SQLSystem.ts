import { SystemType } from "./api";
import { SQLWaypoint } from "./SQLWaypoint";

export interface SystemResp {
  system: SQLSystem;
  waypoints: SQLWaypoint[];
}

export interface SQLSystem {
  sector_symbol: string;
  symbol: string;
  system_type: SystemType;
  x: number;
  y: number;
  waypoints: number | undefined;
}
