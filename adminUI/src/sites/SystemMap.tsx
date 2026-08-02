import { useQuery } from "@apollo/client/react";
import { Button, Dropdown, Spin } from "antd";
import { useState } from "react";
import PageTitle from "../features/PageTitle";
import SystemsMap from "../features/SystemsMap/SystemsMap";
import { GET_SYSTEM_MAP_DATA } from "../graphql/queries";

type MapConfig = {
  minWaypointCount?: number;
  minFleetCount?: number;
  minShipyardCount?: number;
  minMarketplaceCount?: number;
  onlyJumpGates?: "NOT_ACCESSIBLE" | "NONE" | "ACCESSIBLE";
  minShipsHighlighted?: number;
  minFleetsHighlighted?: number;
  minMarketplacesHighlighted?: number;
  minShipyardsHighlighted?: number;
  minWaypointsHighlighted?: number;
};

const defaultConfig: MapConfig = {
  minWaypointCount: 1,
  minFleetCount: 1,
  minShipyardCount: 1,
  minMarketplaceCount: 1,
  onlyJumpGates: "NOT_ACCESSIBLE",
  minShipsHighlighted: 1,
  minFleetsHighlighted: 1,
  minMarketplacesHighlighted: 20,
  minShipyardsHighlighted: 1,
  minWaypointsHighlighted: 100,
};

function SysMap() {
  const { error, data, dataState, refetch, loading } =
    useQuery(GET_SYSTEM_MAP_DATA);
  const [config, setConfig] = useState<MapConfig>(defaultConfig);

  if (dataState != "complete") return <p>Loading... {dataState}</p>;
  if (error) return <p>Error: {error.message}</p>;

  const mapData = {
    systems: data.systems.items.map((system) => ({
      ...system,
      waypoints: system.waypoints.items,
      fleets: system.fleets.items,
    })),
    jumpConnections: data.jumpConnections.items,
  };

  const numOptions = (vals: number[], current: number) =>
    vals.map((v) => ({
      key: String(v),
      label: (
        <span style={{ fontWeight: current === v ? "bold" : "normal" }}>
          {v}
        </span>
      ),
    }));

  const offOption = (current: unknown) => ({
    key: "__off__",
    label: (
      <span
        style={{
          fontWeight: current === undefined ? "bold" : "normal",
          fontStyle: "italic",
        }}
      >
        OFF
      </span>
    ),
  });

  const items = [
    {
      key: "minWaypointCount",
      label: `Min Waypoint Count: ${config.minWaypointCount ?? "OFF"}`,
      children: [
        {
          ...offOption(config.minWaypointCount),
          onClick: () =>
            setConfig((c) => ({ ...c, minWaypointCount: undefined })),
        },
        ...numOptions(
          [0, 1, 5, 10, 30, 50, 80, 100],
          config.minWaypointCount ?? -1,
        ).map((o) => ({
          ...o,
          onClick: () =>
            setConfig((c) => ({ ...c, minWaypointCount: Number(o.key) })),
        })),
      ],
    },
    {
      key: "minFleetCount",
      label: `Min Fleet Count: ${config.minFleetCount ?? "OFF"}`,
      children: [
        {
          ...offOption(config.minFleetCount),
          onClick: () => setConfig((c) => ({ ...c, minFleetCount: undefined })),
        },
        ...numOptions([0, 1, 2, 3, 5, 10], config.minFleetCount ?? -1).map(
          (o) => ({
            ...o,
            onClick: () =>
              setConfig((c) => ({ ...c, minFleetCount: Number(o.key) })),
          }),
        ),
      ],
    },
    {
      key: "minShipyardCount",
      label: `Min Shipyard Count: ${config.minShipyardCount ?? "OFF"}`,
      children: [
        {
          ...offOption(config.minShipyardCount),
          onClick: () =>
            setConfig((c) => ({ ...c, minShipyardCount: undefined })),
        },
        ...numOptions([0, 1, 2, 3, 5, 10], config.minShipyardCount ?? -1).map(
          (o) => ({
            ...o,
            onClick: () =>
              setConfig((c) => ({ ...c, minShipyardCount: Number(o.key) })),
          }),
        ),
      ],
    },
    {
      key: "minMarketplaceCount",
      label: `Min Marketplace Count: ${config.minMarketplaceCount ?? "OFF"}`,
      children: [
        {
          ...offOption(config.minMarketplaceCount),
          onClick: () =>
            setConfig((c) => ({ ...c, minMarketplaceCount: undefined })),
        },
        ...numOptions(
          [0, 1, 2, 5, 10, 20, 25],
          config.minMarketplaceCount ?? -1,
        ).map((o) => ({
          ...o,
          onClick: () =>
            setConfig((c) => ({ ...c, minMarketplaceCount: Number(o.key) })),
        })),
      ],
    },
    {
      key: "onlyJumpGates",
      label: `Only Jump Gates: ${config.onlyJumpGates ?? "OFF"}`,
      children: [
        ...[
          { value: "NOT_ACCESSIBLE" as const, label: "Not Accessible" },
          { value: "NONE" as const, label: "None" },
          { value: "ACCESSIBLE" as const, label: "Accessible" },
        ].map((opt) => ({
          key: opt.value,
          label: (
            <span
              style={{
                fontWeight:
                  config.onlyJumpGates === opt.value ? "bold" : "normal",
              }}
            >
              {opt.label}
            </span>
          ),
          onClick: () => setConfig((c) => ({ ...c, onlyJumpGates: opt.value })),
        })),
      ],
    },
    {
      key: "minShipsHighlighted",
      label: `Min Ships Highlighted: ${config.minShipsHighlighted ?? "OFF"}`,
      children: [
        {
          ...offOption(config.minShipsHighlighted),
          onClick: () =>
            setConfig((c) => ({ ...c, minShipsHighlighted: undefined })),
        },
        ...numOptions(
          [0, 1, 2, 3, 5, 10, 20, 40],
          config.minShipsHighlighted ?? -1,
        ).map((o) => ({
          ...o,
          onClick: () =>
            setConfig((c) => ({ ...c, minShipsHighlighted: Number(o.key) })),
        })),
      ],
    },
    {
      key: "minFleetsHighlighted",
      label: `Min Fleets Highlighted: ${config.minFleetsHighlighted ?? "OFF"}`,
      children: [
        {
          ...offOption(config.minFleetsHighlighted),
          onClick: () =>
            setConfig((c) => ({ ...c, minFleetsHighlighted: undefined })),
        },
        ...numOptions(
          [0, 1, 2, 3, 4, 5],
          config.minFleetsHighlighted ?? -1,
        ).map((o) => ({
          ...o,
          onClick: () =>
            setConfig((c) => ({ ...c, minFleetsHighlighted: Number(o.key) })),
        })),
      ],
    },
    {
      key: "minMarketplacesHighlighted",
      label: `Min Marketplaces Highlighted: ${config.minMarketplacesHighlighted ?? "OFF"}`,
      children: [
        {
          ...offOption(config.minMarketplacesHighlighted),
          onClick: () =>
            setConfig((c) => ({ ...c, minMarketplacesHighlighted: undefined })),
        },
        ...numOptions(
          [0, 1, 5, 10, 20, 25, 30, 40],
          config.minMarketplacesHighlighted ?? -1,
        ).map((o) => ({
          ...o,
          onClick: () =>
            setConfig((c) => ({
              ...c,
              minMarketplacesHighlighted: Number(o.key),
            })),
        })),
      ],
    },
    {
      key: "minShipyardsHighlighted",
      label: `Min Shipyards Highlighted: ${config.minShipyardsHighlighted ?? "OFF"}`,
      children: [
        {
          ...offOption(config.minShipyardsHighlighted),
          onClick: () =>
            setConfig((c) => ({ ...c, minShipyardsHighlighted: undefined })),
        },
        ...numOptions(
          [0, 1, 2, 3, 5, 10],
          config.minShipyardsHighlighted ?? -1,
        ).map((o) => ({
          ...o,
          onClick: () =>
            setConfig((c) => ({
              ...c,
              minShipyardsHighlighted: Number(o.key),
            })),
        })),
      ],
    },
    {
      key: "minWaypointsHighlighted",
      label: `Min Waypoints Highlighted: ${config.minWaypointsHighlighted ?? "OFF"}`,
      children: [
        {
          ...offOption(config.minWaypointsHighlighted),
          onClick: () =>
            setConfig((c) => ({ ...c, minWaypointsHighlighted: undefined })),
        },
        ...numOptions(
          [0, 1, 10, 30, 50, 80, 100],
          config.minWaypointsHighlighted ?? -1,
        ).map((o) => ({
          ...o,
          onClick: () =>
            setConfig((c) => ({
              ...c,
              minWaypointsHighlighted: Number(o.key),
            })),
        })),
      ],
    },
    { type: "divider" as const },
    {
      key: "reset",
      label: (
        <Button block onClick={() => setConfig(defaultConfig)}>
          Reset to Default
        </Button>
      ),
    },
    {
      key: "refetch",
      label: (
        <Button block onClick={() => refetch()}>
          Refetch
        </Button>
      ),
    },
  ];

  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      <PageTitle title={`Systems Map`} />
      <Dropdown menu={{ items }} trigger={["contextMenu"]}>
        <div style={{ width: "100%", height: "100%", position: "relative" }}>
          <SystemsMap
            zoomMax={10000}
            zoomMin={0.5}
            data={mapData}
            config={config}
          />
          {loading && (
            <span className="absolute inset-0 bg-gray-800/50 flex items-center justify-center">
              <Spin spinning={true} />
            </span>
          )}
        </div>
      </Dropdown>
    </div>
  );
}

export default SysMap;
