import { TradeSymbol } from "./api";

export interface TradeRoute {
  id: number;
  symbol: TradeSymbol;
  ship_symbol: string;
  purchase_waypoint: string;
  sell_waypoint: string;
  finished: boolean;
  trade_volume: number;
  predicted_purchase_price: number;
  predicted_sell_price: number;
  sum: number;
  expenses: number;
  income: number;
  profit: number;
}
