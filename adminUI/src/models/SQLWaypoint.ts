import {
  FactionSymbol,
  MarketTradeGoodTypeEnum,
  TradeSymbol,
  WaypointModifierSymbol,
  WaypointTraitSymbol,
  WaypointType,
} from "./api";
import { ConstructionMaterial } from "./Construction";
import { MarketTrade, MarketTradeGood } from "./Market";
import {
  ShipTransaction,
  Shipyard,
  ShipyardShip,
  ShipyardShipType,
} from "./Shipyard";
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
  trade_goods?: {
    symbol: TradeSymbol;
    type: MarketTradeGoodTypeEnum;
  }[];
}

export interface WaypointResponse {
  jump_gate_connections?: JumpGateConnection[];
  market_trade_goods?: MarketTradeGood[];
  market_trades?: MarketTrade[];
  transactions?: Transaction[];
  constructions?: ConstructionMaterial[];
  trade_good_history?: MarketTradeGood[];
  ship_transactions?: ShipTransaction[];
  ship_types?: ShipyardShipType[];
  ships?: ShipyardShip[];
  shipyard?: Shipyard;
  waypoint: SQLWaypoint;
}

export interface JumpGateConnection {
  created_at: string;
  from: string;
  id: number;
  to: string;
  updated_at: string;
}
