import {
  ActivityLevel,
  MarketTradeGoodTypeEnum,
  SupplyLevel,
  TradeSymbol,
} from "./api";

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
