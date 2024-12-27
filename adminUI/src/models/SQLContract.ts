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

interface ContractResponse {
  0: string;
  1: SQLContract;
  2: ContractDeliverable[];
  3: Transaction[];
}

export type { ContractDeliverable, ContractResponse, SQLContract };
