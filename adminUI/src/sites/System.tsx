import { Button, Divider, Flex, Space, Table, TableProps, Tooltip } from "antd";
import { Link, useParams } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import {
  Waypoint,
  WaypointModifier,
  WaypointModifierSymbol,
  WaypointOrbital,
  WaypointTrait,
  WaypointTraitSymbol,
  WaypointType,
} from "../models/api";
import useMyStore, { backendUrl } from "../store";

function System() {
  const { systemID } = useParams();
  const Waypoints = useMyStore((state) => state.waypoints[systemID || ""]);
  const setWaypoints = useMyStore((state) => state.setWaypoints);

  const columns: TableProps<Waypoint>["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Type",
      dataIndex: "type",
      key: "type",
      sorter: (a, b) => a.type.localeCompare(b.type),
      filters: Object.values(WaypointType).map((type) => ({
        text: type,
        value: type,
      })),
      onFilter: (value, record) => record.type === value,
    },
    {
      title: "System Symbol",
      dataIndex: "systemSymbol",
      key: "systemSymbol",
      render: (systemSymbol: string) => (
        <Link to={`/system/${systemSymbol}`}>{systemSymbol}</Link>
      ),
      sorter: (a, b) => a.systemSymbol.localeCompare(b.systemSymbol),
    },
    {
      title: "Position (X)",
      dataIndex: "x",
      key: "x",
      sorter: (a, b) => a.x - b.x,
    },
    {
      title: "Position (Y)",
      dataIndex: "y",
      key: "y",
      sorter: (a, b) => a.y - b.y,
    },
    {
      title: "Orbitals",
      dataIndex: "orbitals",
      key: "orbitals",
      render: (orbitals) =>
        orbitals.length > 0 ? (
          <Flex gap={1} vertical>
            {orbitals.map((o: WaypointOrbital) => (
              <WaypointLink waypoint={o.symbol}>{o.symbol}</WaypointLink>
            ))}
          </Flex>
        ) : (
          "None"
        ), // List symbols of orbitals or "None"
      sorter: (a, b) => a.orbitals.length - b.orbitals.length,
    },
    {
      title: "Orbits",
      dataIndex: "orbits",
      key: "orbits",
      render: (orbits) =>
        orbits ? (
          <WaypointLink waypoint={orbits}>{orbits}</WaypointLink>
        ) : (
          "N/A"
        ), // Display "N/A" if undefined
      sorter: (a, b) => (a.orbits ?? "").localeCompare(b.orbits ?? ""),
    },
    {
      title: "Faction",
      dataIndex: "faction",
      key: "faction",
      render: (faction) => (faction ? faction.symbol : "N/A"), // Display faction symbol or "N/A"
      sorter: (a, b) =>
        (a.faction?.symbol ?? "").localeCompare(b.faction?.symbol ?? ""),
    },
    {
      title: "Traits",
      dataIndex: "traits",
      key: "traits",
      render: (traits) => (
        <Flex gap={1} vertical>
          {traits.map((trait: WaypointTrait) => (
            <Tooltip
              key={trait.symbol}
              title={`${trait.symbol} - ${trait.description}`}
            >
              <span>{trait.name}</span>
            </Tooltip>
          ))}
        </Flex>
      ), // List names of traits
      sorter: (a, b) => a.traits.length - b.traits.length,
      filters: Object.values(WaypointTraitSymbol).map((trait) => ({
        text: trait,
        value: trait,
      })),
      onFilter: (value, record) =>
        record.traits.some((t) => t.symbol === value),
    },
    {
      title: "Modifiers",
      dataIndex: "modifiers",
      key: "modifiers",
      render: (modifiers) =>
        modifiers && modifiers.length > 0 ? (
          <span>
            {modifiers?.map((modifier: WaypointModifier) => (
              <Tooltip
                key={modifier.symbol}
                title={`${modifier.symbol} - ${modifier.description}`}
              >
                <span>{modifier.name}</span>
              </Tooltip>
            ))}
          </span>
        ) : (
          "None"
        ),
      sorter: (a, b) => (a.modifiers?.length ?? 0) - (b.modifiers?.length ?? 0),
      filters: Object.values(WaypointModifierSymbol).map((modifier) => ({
        text: modifier,
        value: modifier,
      })),
      onFilter: (value, record) =>
        record.modifiers?.some((m) => m.symbol === value) ?? false,
    },
    {
      title: "Chart",
      dataIndex: "chart",
      key: "chart",
      render: (chart) => (chart ? chart.submittedBy : "N/A"), // Display chart symbol or "N/A"
    },
    {
      title: "Under Construction",
      dataIndex: "isUnderConstruction",
      key: "isUnderConstruction",
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`System ${systemID}`} />
      <h2>System {systemID}</h2>
      <Space>
        <Link to={`/map/system/${systemID}`}>Map</Link>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/waypoints`)
              .then((response) => response.json())
              .then(setWaypoints);
          }}
        >
          Reload
        </Button>
      </Space>
      <Divider />
      <Table
        columns={columns}
        dataSource={Object.values(Waypoints)}
        rowKey={"symbol"}
      />
    </div>
  );
}

export default System;
