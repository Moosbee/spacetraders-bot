// Enum definitions
enum RequestedShipType {
  Scrapper = "Scrapper", // Used for market scrapping does not need speed, cargo or anything, the cheapest ship will do
  Explorer = "Explorer", // ship with a warp drive
  Probe = "Probe", // a ship with no fuel thus infinite range, but nothing else
  Transporter = "Transporter", // ship with at least 40 units of cargo and decent range
  Mining = "Mining", // ship equipped with extractor setup and cargo
  Siphon = "Siphon", // ship equipped with siphon setup and cargo
  Survey = "Survey", // ship equipped with survey setup does not need cargo
}

enum Priority {
  High = 100_000,
  Medium = 500_000,
  Low = 1_000_000,
}

enum Budget {
  VeryHigh = 10_000_000,
  High = 1_000_000,
  Medium = 500_000,
  Low = 100_000,
}

// Type for the ship requirements
type ShipRequirement = [RequestedShipType, Priority, Budget];

// Required ships interface
interface RequiredShips {
  ships: Record<string, ShipRequirement[]>;
}

// Interface for the API response
interface ShipManagementResponse {
  chart: RequiredShips;
  construction: RequiredShips;
  contract: RequiredShips;
  mining: RequiredShips;
  scrap: RequiredShips;
  trading: RequiredShips;
}

export { Budget, Priority, RequestedShipType };
export type { RequiredShips, ShipManagementResponse, ShipRequirement };
