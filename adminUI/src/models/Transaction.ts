import { MarketTransactionTypeEnum, TradeSymbol } from "./api";

export interface Transaction {
  waypoint_symbol: string;
  ship_symbol: string;
  trade_symbol: TradeSymbol;
  type: MarketTransactionTypeEnum;
  units: number;
  price_per_unit: number;
  total_price: number;
  timestamp: string;
  contract: string | null;
  trade_route: number | null;
  mining: string | null;
}
