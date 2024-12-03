import { Button, Space, Switch, Table, TableProps } from "antd";
import { useState } from "react";
import { Link } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import Timer from "../features/Timer/Timer";
import { ShipNavFlightMode, ShipNavStatus, ShipRole } from "../models/api";
import RustShip, { SystemShipRole, SystemShipRoles } from "../models/ship";
import useMyStore, { backendUrl } from "../store";

function Ships() {
  const ships = useMyStore((state) => state.ships);
  const setShips = useMyStore((state) => state.setShips);

  const [showCooldown, setShowCooldown] = useState(false);

  // useEffect(() => {
  //   fetch(`http://${backendUrl}/ships`)
  //     .then((response) => response.json())
  //     .then(setShips);
  // }, [setShips]);

  const columns: TableProps<RustShip>["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      render: (symbol) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) =>
        Number.parseInt(a.symbol.split("-")[1], 16) -
        Number.parseInt(b.symbol.split("-")[1], 16),
    },
    {
      title: "Role",
      dataIndex: "role",
      key: "role",
      filters: Object.values(SystemShipRoles).map((role) => ({
        text: role,
        value: role,
      })),
      render: (role: SystemShipRole) =>
        `${role.type} ${
          role.type === "Contract"
            ? role.data === null
              ? ""
              : role.data[0] + " (" + role.data[1] + ")"
            : ""
        }${
          role.type === "Trader"
            ? role.data === null
              ? ""
              : role.data[0] + " (" + role.data[1] + ")"
            : ""
        }`,
      onFilter: (value, record) => record.role.type === value,
      sorter: (a, b) => a.role.type.localeCompare(b.role.type),
    },
    {
      title: "Registration Role",
      dataIndex: "registration_role",
      key: "registration_role",
      filters: Object.values(ShipRole).map((role) => ({
        text: role,
        value: role,
      })),
      onFilter: (value, record) => record.registration_role === value,
      sorter: (a, b) => a.registration_role.localeCompare(b.registration_role),
    },

    {
      title: "Current Waypoint",
      dataIndex: ["nav", "waypoint_symbol"],
      key: "current_waypoint",
      sorter: (a, b) =>
        a.nav.waypoint_symbol.localeCompare(b.nav.waypoint_symbol),
      render: (value: string, record) => (
        <span>
          <Link to={`/system/${record.nav.system_symbol}`}>
            {record.nav.system_symbol}
          </Link>
          <Link to={`/system/${record.nav.system_symbol}/${value}`}>
            {record.nav.waypoint_symbol.replace(record.nav.system_symbol, "")}
          </Link>
        </span>
      ),
    },
    {
      title: "Flight Mode",
      dataIndex: ["nav", "flight_mode"],
      key: "flight_mode",
      filters: Object.values(ShipNavFlightMode).map((role) => ({
        text: role,
        value: role,
      })),
      onFilter: (value, record) => record.nav.flight_mode === value,
      sorter: (a, b) => a.nav.flight_mode.localeCompare(b.nav.flight_mode),
    },
    {
      title: "Navigation Status",
      dataIndex: ["nav", "status"],
      key: "nav_status",
      render: (value: ShipNavStatus, record) => (
        <span>
          {value}
          <br />
          {value === "IN_TRANSIT" && (
            <span>
              {record.nav.route.origin_symbol} -{">"}{" "}
              {record.nav.route.destination_symbol}
              (<Timer time={record.nav.route.arrival} />)
            </span>
          )}
        </span>
      ),
      filters: Object.values(ShipNavStatus).map((status) => ({
        text: status,
        value: status,
      })),
      onFilter: (value, record) => record.nav.status === value,
      sorter: (a, b) => a.nav.status.localeCompare(b.nav.status),
    },

    {
      title: "Autopilot",
      key: "autopilot",
      render: (_value, record) => (
        <>
          {record.nav.auto_pilot && (
            <span>
              {record.nav.auto_pilot.origin_symbol} -{">"}{" "}
              {record.nav.auto_pilot.destination_symbol} (
              <Timer time={record.nav.auto_pilot.arrival} />)
            </span>
          )}
        </>
      ),
    },
    {
      title: "Engine Speed",
      dataIndex: "engine_speed",
      key: "engine_speed",
      sorter: (a, b) => a.engine_speed - b.engine_speed,
      align: "right",
    },
    {
      title: "Cargo",
      dataIndex: ["cargo", "units"],
      key: "cargo_units",
      render: (value: number, record) => `${value} / ${record.cargo.capacity}`,
      align: "right",
      sorter: (a, b) => a.cargo.capacity - b.cargo.capacity,
    },
    {
      title: "Fuel",
      dataIndex: ["fuel", "current"],
      key: "fuel_current",
      render: (value: number, record) => `${value} / ${record.fuel.capacity}`,
      align: "right",
      sorter: (a, b) => a.fuel.capacity - b.fuel.capacity,
    },

    ...(showCooldown
      ? [
          {
            title: "Cooldown",
            dataIndex: "cooldown_expiration",
            key: "cooldown_expiration",
            render: (value: string | null) =>
              value && new Date(value).toLocaleString(),
          },
        ]
      : []),
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="All Ships" />
      <Space>
        <h2>All Ships</h2>
        <Button onClick={() => setShips({})}>Reset</Button>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/ships`)
              .then((response) => response.json())
              .then(setShips);
          }}
        >
          Refresh
        </Button>
        <Switch
          checked={showCooldown}
          onChange={(checked) => setShowCooldown(checked)}
        />
        Show Cooldown
      </Space>
      <Table
        dataSource={Object.values(ships)}
        columns={columns}
        rowKey={"symbol"}
      />
    </div>
  );
}

export default Ships;
