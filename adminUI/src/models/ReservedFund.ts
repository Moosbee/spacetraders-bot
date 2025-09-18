interface ReservedFund {
  id: number;
  amount: number;
  status: FundStatus;
  actual_amount: number;
  created_at: string; // ISO 8601 datetime string
  updated_at: string; // ISO 8601 datetime string
}

// FundStatus enum based on your Rust definition
enum FundStatus {
  Reserved = "RESERVED",
  Used = "USED",
  Cancelled = "CANCELLED",
}

// BudgetInfo interface
interface BudgetInfo {
  current_funds: number;
  iron_reserve: number;
  reserved_amount: number;
  spendable: number;
  reservations: ReservedFund[];
}

// The main response interface for your warp JSON response
interface BudgetResponse {
  budget_info: BudgetInfo;
  all_reservations: ReservedFund[];
}

export { FundStatus };
export type { BudgetInfo, BudgetResponse, ReservedFund };
