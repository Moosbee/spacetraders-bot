import {
  Button,
  Descriptions,
  Divider,
  Flex,
  Space,
  Table,
  TableProps,
} from "antd";
import { Link, useParams } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import {
  WaypointModifierSymbol,
  WaypointTraitSymbol,
  WaypointType,
} from "../models/api";
import { SQLSystem } from "../models/SQLSystem";
import { SQLWaypoint } from "../models/SQLWaypoint";
import useMyStore, { backendUrl } from "../store";

function System() {
  const { systemID } = useParams();
  const System:
    | {
        system: SQLSystem;
        waypoints: SQLWaypoint[];
      }
    | undefined = useMyStore((state) => state.systems[systemID || ""]);
  const setWaypoints = useMyStore((state) => state.setSystem);

  const Waypoints = System?.waypoints || [];

  const items = [
    {
      label: "Symbol",
      key: "symbol",
      children: System?.system?.symbol,
    },
    {
      label: "Sector Symbol",
      key: "sectorSymbol",
      children: System?.system?.sector_symbol,
    },
    {
      key: "reload",
      children: (
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/systems/${systemID}`)
              .then((response) => response.json())
              .then((data) => {
                const system = data.system;
                const waypoints = data.waypoints;
                setWaypoints(system, waypoints);
              });
          }}
        >
          Reload
        </Button>
      ),
    },
    {
      label: "System Type",
      key: "systemType",
      children: System?.system?.system_type,
    },
    {
      label: "Waypoints",
      key: "Waypoints",
      children: System?.waypoints?.length,
    },
    {
      key: "Map",
      children: <Link to={`/map/system/${systemID}`}>Map</Link>,
    },
    {
      label: "X Coordinate",
      key: "x",
      children: System?.system?.x,
    },
    {
      label: "Y Coordinate",
      key: "y",
      children: System?.system?.y,
    },

    {
      key: "placeholder",
      children: "",
    },
  ];

  const columns: TableProps<SQLWaypoint>["columns"] = [
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
      dataIndex: "waypoint_type",
      key: "waypoint_type",
      sorter: (a, b) => a.waypoint_type.localeCompare(b.waypoint_type),
      filters: Object.values(WaypointType).map((type) => ({
        text: type,
        value: type,
      })),
      onFilter: (value, record) => record.waypoint_type === value,
    },
    {
      title: "System Symbol",
      dataIndex: "system_symbol",
      key: "system_symbol",
      render: (system_symbol) => (
        <Link to={`/system/${system_symbol}`}>{system_symbol}</Link>
      ),
      sorter: (a, b) => a.system_symbol.localeCompare(b.system_symbol),
    },
    {
      title: "Pos X",
      dataIndex: "x",
      key: "x",
      sorter: (a, b) => a.x - b.x,
    },
    {
      title: "Pos Y",
      dataIndex: "y",
      key: "y",
      sorter: (a, b) => a.y - b.y,
    },
    {
      title: "Orbitals",
      dataIndex: "orbitals",
      key: "orbitals",
      render: (orbitals: string[]) =>
        orbitals.length > 0 ? (
          <Flex gap={1} vertical>
            {orbitals.map((symbol) => (
              <WaypointLink waypoint={symbol} key={symbol}>
                {symbol}
              </WaypointLink>
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
      render: (faction) => (faction ? faction : "N/A"), // Display faction symbol or "N/A"
      sorter: (a, b) => (a.faction ?? "").localeCompare(b.faction ?? ""),
    },
    {
      title: "Traits",
      dataIndex: "traits",
      key: "traits",
      render: (traits) => (
        <Flex gap={1} vertical>
          {traits.map((trait: WaypointTraitSymbol) => (
            <span>{trait}</span>
          ))}
        </Flex>
      ), // List names of traits
      sorter: (a, b) => a.traits.length - b.traits.length,
      filters: Object.values(WaypointTraitSymbol).map((trait) => ({
        text: trait,
        value: trait,
      })),
      onFilter: (value, record) => record.traits.some((t) => t === value),
    },
    {
      title: "Modifiers",
      dataIndex: "modifiers",
      key: "modifiers",
      render: (modifiers) =>
        modifiers && modifiers.length > 0 ? (
          <span>
            {modifiers?.map((modifier: WaypointModifierSymbol) => (
              <span>{modifier}</span>
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
        record.modifiers?.some((m) => m === value) ?? false,
    },
    {
      title: "Chart by",
      dataIndex: "charted_by",
      key: "charted_by",
      render: (charted_by) => (charted_by ? charted_by : "N/A"), // Display chart symbol or "N/A"
      sorter: (a, b) => (a.charted_by ?? "").localeCompare(b.charted_by ?? ""),
    },
    {
      title: "Chart on",
      dataIndex: "charted_on",
      key: "charted_on",
      render: (charted_on) =>
        charted_on ? new Date(charted_on).toLocaleString() : "N/A", // Display chart symbol or "N/A"
      sorter: (a, b) => (a.charted_on ?? "").localeCompare(b.charted_on ?? ""),
    },
    {
      title: "Construction",
      dataIndex: "is_under_construction",
      key: "is_under_construction",
      render: (value) => (value ? "Yes" : "No"), // Render boolean as "Yes" or "No"
      sorter: (a, b) =>
        (a.is_under_construction ? 1 : 0) - (b.is_under_construction ? 1 : 0),
      filters: [
        { text: "Yes", value: true },
        { text: "No", value: false },
      ],
      onFilter: (value, record) => record.is_under_construction === value,
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`System ${systemID}`} />
      <h2>System {systemID}</h2>
      <Space>
        <Descriptions bordered column={3} items={items} />
      </Space>
      <Divider />
      <Table
        columns={columns}
        dataSource={Waypoints || []}
        rowKey={(row) => row.symbol}
        pagination={{
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
    </div>
  );
}

export default System;
