import { GetFleetsQuery } from "../../gql/graphql";

type FleetRecord = GetFleetsQuery["fleets"]["items"][number];
type FleetConfig = FleetRecord["config"];

function configToEntries(config: FleetConfig): [string, unknown][] {
  return Object.entries(config as unknown as Record<string, unknown>).filter(
    ([key]) => key !== "__typename",
  );
}

function formatConfigValue(value: unknown): string {
  if (Array.isArray(value)) return value.join(", ");
  if (value === null || value === undefined) return "";
  return String(value);
}

function configSummary(config: FleetConfig): string {
  switch (config.__typename) {
    case "ChartingConfig":
      return config.chartOnlyJumpGates
        ? `Charting: Gates (${config.chartingProbeCount} probes)`
        : `Charting: System (${config.chartingProbeCount} probes)`;
    case "ConstructionConfig":
      return `Construction: ${config.constructionMode} @ ${config.constructionWaypoint}`;
    case "ContractConfig":
      return `Contract: ${config.contractShipCount} ships`;
    case "ManuelConfig":
      return `Manuel: ${config.config}`;
    case "MiningConfig":
      return `Mining: ${config.minersPerWaypoint}M/${config.siphonersPerWaypoint}Si/${config.surveyersPerWaypoint}Su per wp (${config.miningWaypoints} wp)`;
    case "ScrapingConfig":
      return `Scraping: ${config.allowedRequests} req, notify=${config.notifyOnShipyard}`;
    case "TradingConfig":
      return `Trading: ${config.tradeMode}`;
    default:
      return (config as { __typename?: string }).__typename ?? "Unknown";
  }
}

export { configSummary, configToEntries, formatConfigValue };
