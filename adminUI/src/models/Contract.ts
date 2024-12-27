export interface Contract {
  id: string;
  faction_symbol: string;
  contract_type: string;
  accepted: boolean;
  fulfilled: boolean;
  deadline_to_accept: string;
  on_accepted: number;
  on_fulfilled: number;
  deadline: string;
  totalprofit: number;
  total_expenses: number;
  net_profit: number;
}
