// Enum for assignment levels
enum AssignLevel {
  Inactive = "Inactive", // Ship is at the waypoint but not active
  OnTheWay = "OnTheWay", // Ship is assigned to a waypoint and is on its way
  Active = "Active", // Ship is assigned to a waypoint and is there
}

// Interface for waypoint information
interface WaypointInfo {
  waypoint_symbol: string;
  assigned_ships: Record<string, AssignLevel>; // ship_symbol -> level
  last_updated: string; // ISO date string from chrono::DateTime<chrono::Utc>
}

// Type for a single assignment (tuple in Rust)
type Assignment = [string, WaypointInfo];

// Interface for the API response
interface MiningAssignmentsResponse {
  assignments: Assignment[];
}

export { AssignLevel };
export type { Assignment, MiningAssignmentsResponse, WaypointInfo };
