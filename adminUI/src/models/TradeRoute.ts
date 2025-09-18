import { TradeSymbol } from "./api";
import { MarketTrade, MarketTradeGood } from "./Market";

export interface TradeRoute {
  id: number;
  symbol: TradeSymbol;
  ship_symbol: string;
  purchase_waypoint: string;
  sell_waypoint: string;
  status: "Delivered" | "InTransit" | "Failed";
  trade_volume: number;
  predicted_purchase_price: number;
  predicted_sell_price: number;
  sum: number;
  expenses: number;
  income: number;
  profit: number;
  reserved_fund?: number;
}

export interface PossibleTrade {
  purchase: MarketTrade;
  purchase_good?: MarketTradeGood;
  sell: MarketTrade;
  sell_good?: MarketTradeGood;
  symbol: TradeSymbol;
}
