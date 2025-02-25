import { ContractTypeEnum, FactionSymbol, TradeSymbol } from "./api";
import { Transaction } from "./Transaction";

interface SQLContract {
  id: string;
  faction_symbol: FactionSymbol;
  contract_type: ContractTypeEnum;
  accepted: boolean;
  fulfilled: boolean;
  deadline_to_accept: string;
  on_accepted: number;
  on_fulfilled: number;
  deadline: string;
}

interface ContractDeliverable {
  contract_id: string;
  trade_symbol: TradeSymbol;
  destination_symbol: string;
  units_required: number;
  units_fulfilled: number;
}

export interface ContractShipment {
  id: number;
  contract_id: string;
  ship_symbol: string;
  trade_symbol: TradeSymbol;
  units: number;
  destination_symbol: string;
  purchase_symbol: string;
  created_at: string;
  updated_at: string;
  status: string;
}

interface ContractResponse {
  0: string;
  1: SQLContract;
  2: ContractDeliverable[];
  3: Transaction[];
  4: ContractShipment[];
}

export type { ContractDeliverable, ContractResponse, SQLContract };
