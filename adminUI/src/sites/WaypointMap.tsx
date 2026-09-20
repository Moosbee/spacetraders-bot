import { useQuery } from "@apollo/client/react";
import { Button, Dropdown, Result, Spin } from "antd";
import { useState } from "react";
import { useParams } from "react-router-dom";
import MapHolder from "../features/MapHolder/MapHolder";
import PageTitle from "../features/PageTitle";
import WaypointMap from "../features/WaypointMap/WaypointMap";
import { GET_SYSTEM_MAP } from "../graphql/queries";

type MapConfig = {
  showAutoPilot: "ALL" | "SELECTED" | "NONE";
  highlightSelectedShip: boolean;
  highlightSelectedWaypoint: boolean;
  showShips: boolean;
};

const defaultConfig: MapConfig = {
  showAutoPilot: "SELECTED",
  highlightSelectedShip: true,
  highlightSelectedWaypoint: true,
  showShips: true,
};

function WpMap() {
  const { systemID } = useParams();
  const { loading, error, data, dataState, refetch } = useQuery(
    GET_SYSTEM_MAP,
    {
      variables: { systemSymbol: systemID || "" },
    },
  );
  const [config, setConfig] = useState<MapConfig>(defaultConfig);

  const boldIf = (active: boolean) => ({
    fontWeight: active ? "bold" : "normal",
  });

  const booleanOptions = (current: boolean, update: (value: boolean) => void) =>
    (
      [
        { key: "ON", value: true },
        { key: "OFF", value: false },
      ] as const
    ).map(({ key, value }) => ({
      key,
      label: <span style={boldIf(current === value)}>{key}</span>,
      onClick: () => update(value),
    }));

  const items = [
    {
      key: "showAutoPilot",
      label: `Auto Pilot Routes: ${config.showAutoPilot}`,
      children: (["ALL", "SELECTED", "NONE"] as const).map((value) => ({
        key: value,
        label: (
          <span style={boldIf(config.showAutoPilot === value)}>{value}</span>
        ),
        onClick: () => setConfig((c) => ({ ...c, showAutoPilot: value })),
      })),
    },
    {
      key: "highlightSelectedShip",
      label: `Highlight Selected Ship: ${config.highlightSelectedShip ? "ON" : "OFF"}`,
      children: booleanOptions(config.highlightSelectedShip, (value) =>
        setConfig((c) => ({ ...c, highlightSelectedShip: value })),
      ),
    },
    {
      key: "highlightSelectedWaypoint",
      label: `Highlight Selected Waypoint: ${config.highlightSelectedWaypoint ? "ON" : "OFF"}`,
      children: booleanOptions(config.highlightSelectedWaypoint, (value) =>
        setConfig((c) => ({ ...c, highlightSelectedWaypoint: value })),
      ),
    },
    {
      key: "showShips",
      label: `Show Ships: ${config.showShips ? "ON" : "OFF"}`,
      children: booleanOptions(config.showShips, (value) =>
        setConfig((c) => ({ ...c, showShips: value })),
      ),
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
      <PageTitle title={`${systemID} Map`} />
      {/* <div > */}
      <Spin spinning={loading} fullscreen />
      {dataState == "complete" && !error && (
        <Dropdown menu={{ items }} trigger={["contextMenu"]}>
          <div>
            <MapHolder>
              <WaypointMap
                systemData={data.system}
                systemWaypoints={data.system.waypoints.items}
                systemShips={data.system.ships}
                config={config}
              />
            </MapHolder>
          </div>
        </Dropdown>
      )}
      {error && (
        <Result
          status="error"
          title="Failed to load system map"
          subTitle="Please check your network connection or try again later."
          extra={[
            <Button key="refetch" onClick={() => refetch()} type="primary">
              Try again
            </Button>,
          ]}
        ></Result>
      )}
    </div>
  );
}

export default WpMap;
