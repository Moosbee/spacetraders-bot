import {
  ActivityLevel,
  FactionSymbol,
  MarketTradeGoodTypeEnum,
  SupplyLevel,
  TradeSymbol,
  WaypointModifierSymbol,
  WaypointTraitSymbol,
  WaypointType,
} from "./api";
import { Transaction } from "./Transaction";

export interface SQLWaypoint {
  charted_by: string;
  charted_on: string;
  created_at: string;
  faction?: FactionSymbol;
  is_under_construction: boolean;
  modifiers: WaypointModifierSymbol[];
  orbitals: string[];
  orbits?: string;
  symbol: string;
  system_symbol: string;
  traits: WaypointTraitSymbol[];
  waypoint_type: WaypointType;
  x: number;
  y: number;
  unstable_since?: string;
}

export interface WaypointResponse {
  market_trade_goods: MarketTradeGood[];
  market_trades: MarketTrade[];
  transactions: Transaction[];
  waypoint: SQLWaypoint;
}

export interface MarketTradeGood {
  activity?: ActivityLevel;
  created: string;
  created_at: string;
  purchase_price: number;
  sell_price: number;
  supply: SupplyLevel;
  symbol: TradeSymbol;
  trade_volume: number;
  type: MarketTradeGoodTypeEnum;
  waypoint_symbol: string;
}

export interface MarketTrade {
  created_at: string;
  symbol: TradeSymbol;
  type: MarketTradeGoodTypeEnum;
  waypoint_symbol: string;
}
