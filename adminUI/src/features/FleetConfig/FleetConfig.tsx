import { Descriptions, DescriptionsProps } from "antd";
import { FleetConfig } from "../../gql/graphql";

function FleetConfigDetails({ config }: { config: FleetConfig }) {
  let items: DescriptionsProps["items"] = [
    {
      label: "Type",
      children: config.__typename,
    },
  ];

  switch (config.__typename) {
    case "ChartingConfig":
      items = [
        {
          label: "Chart Only Jump Gates",
          children: config.chartOnlyJumpGates ? "Yes" : "No",
        },
        { label: "Charting Probe Count", children: config.chartingProbeCount },
      ];
      break;
    case "ConstructionConfig":
      items = [
        { label: "Construction Mode", children: config.constructionMode },
        {
          label: "Construction Ship Count",
          children: config.constructionShipCount,
        },
        {
          label: "Construction Waypoint",
          children: config.constructionWaypoint,
        },
      ];
      break;
    case "ContractConfig":
      items = [
        { label: "Contract Ship Count", children: config.contractShipCount },
      ];
      break;
    case "ManuelConfig":
      items = [{ label: "Config", children: config.config }];
      break;
    case "MiningConfig":
      items = [
        { label: "Mining Waypoints", children: config.miningWaypoints },
        {
          label: "Ignore Engineered Asteroids",
          children: config.ignoreEngineeredAsteroids ? "Yes" : "No",
        },
        {
          label: "Stop All Unstable",
          children: config.stopAllUnstable ? "Yes" : "No",
        },
        {
          label: "Unstable Since Timeout",
          children: config.unstableSinceTimeout,
        },
        { label: "Syphon Waypoints", children: config.syphonWaypoints },
        {
          label: "Minimum Transporter Cargo Space",
          children: config.minTransporterCargoSpace,
        },
        {
          label: "Minimum Mining Cargo Space",
          children: config.minMiningCargoSpace,
        },
        {
          label: "Minimum Siphon Cargo Space",
          children: config.minSiphonCargoSpace,
        },
        {
          label: "Surveyers Per Waypoint",
          children: config.surveyersPerWaypoint,
        },
        { label: "Miners Per Waypoint", children: config.minersPerWaypoint },
        {
          label: "Transporters Per Waypoint",
          children: config.miningTransportersPerWaypoint,
        },
        {
          label: "Siphoners Per Waypoint",
          children: config.siphonersPerWaypoint,
        },
        {
          label: "Mining Eject List",
          children: (
            <div>
              {config.miningEjectList.toSorted().map((e) => (
                <div key={e}>{e}</div>
              ))}
            </div>
          ),
        },
        {
          label: "Mining Prefer List",
          children: (
            <div>
              {config.miningPreferList.toSorted().map((e) => (
                <div key={e}>{e}</div>
              ))}
            </div>
          ),
        },
      ];
      break;
    case "ScrapingConfig":
      items = [
        { label: "Ship Market Ratio", children: config.shipMarketRatio },
        { label: "Allowed Requests", children: config.allowedRequests },
        {
          label: "Notify on Shipyard",
          children: config.notifyOnShipyard ? "Yes" : "No",
        },
      ];
      break;
    case "TradingConfig":
      items = [
        { label: "Trade Mode", children: config.tradeMode },
        {
          label: "Trade Profit Threshold",
          children: config.tradeProfitThreshold,
        },
        { label: "Minimum Cargo Space", children: config.minCargoSpace },
        { label: "Ship Market Ratio", children: config.shipMarketRatio },
        { label: "Purchase Multiplier", children: config.purchaseMultiplier },

        {
          label: "Market Blacklist",
          children: (
            <div>
              {config.marketBlacklist.toSorted().map((e) => (
                <div key={e}>{e}</div>
              ))}
            </div>
          ),
        },
        {
          label: "Market Prefer List",
          children: (
            <div>
              {config.marketPreferList.toSorted().map((e) => (
                <div key={e}>{e}</div>
              ))}
            </div>
          ),
        },
      ];
      break;
    default:
      items = [
        {
          label: "Config",
          children: "No Config Found",
        },
      ];
  }

  return (
    <Descriptions
      bordered
      size="small"
      column={4}
      items={items}
      title={`Config (${config.__typename})`}
    />
  );
}

export default FleetConfigDetails;
