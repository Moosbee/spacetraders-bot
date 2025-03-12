import { SystemType } from "./api";
import { MarketTrade, MarketTradeGood } from "./Market";
import { SQLWaypoint } from "./SQLWaypoint";

export interface SystemResp {
  system: SQLSystem;
  waypoints: {
    waypoint: SQLWaypoint;
    trade_goods: MarketTrade[];
    market_trade_goods: MarketTradeGood[];
  }[];
}

export interface SQLSystem {
  sector_symbol: string;
  symbol: string;
  system_type: SystemType;
  x: number;
  y: number;
  waypoints: number | undefined;
}
