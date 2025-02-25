import { Button, Flex, Popover, Space, Switch, Table, TableProps } from "antd";
import { useState } from "react";
import { Link } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import RoleRenderer from "../features/RoleRenderer/RoleRenderer";
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
      defaultSortOrder: "ascend",
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
      render: (role: SystemShipRole, record) => (
        <RoleRenderer role={role} status={record.status} />
      ),
      defaultFilteredValue: [
        "Construction",
        "Trader",
        "TempTrader",
        "Contract",
        "Mining",
        "Manuel",
      ],
      onFilter: (value, record) => record.role === value,
      sorter: (a, b) => {
        const num = a.role.localeCompare(b.role);
        if (num === 0) {
          if (a.status.type === "Mining" && b.status.type === "Mining") {
            const data_a = a.status.data ?? "";
            const data_b = b.status.data ?? "";
            return data_a.localeCompare(data_b);
          }
        }
        return num;
      },
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
          {value === "IN_TRANSIT" && (
            <>
              {" "}
              (<Timer time={record.nav.route.arrival} />)
              <br />
              <span>
                {record.nav.route.origin_symbol} -{">"}{" "}
                {record.nav.route.destination_symbol}
              </span>
            </>
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
      align: "center",
      render: (_value, record) => (
        <>
          {record.nav.auto_pilot && (
            <span>
              {record.nav.auto_pilot.origin_symbol} -{">"}{" "}
              {record.nav.auto_pilot.destination_symbol}
              <br />(
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
      render: (value: number, record) => (
        <Popover
          content={
            <Flex vertical>
              {Object.entries(record.cargo.inventory).map((item) => (
                <Flex gap={6} justify="space-between">
                  <span>{item[0]}</span>
                  <span>{item[1]}</span>
                </Flex>
              ))}
            </Flex>
          }
        >
          {`${value} / ${record.cargo.capacity}`}
        </Popover>
      ),
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
            render: (value: string | null) => value && <Timer time={value} />,
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
        rowKey={(ship) => ship.symbol}
        pagination={{
          showSizeChanger: true,
          pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
          defaultPageSize: 100,
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
    </div>
  );
}

export default Ships;
