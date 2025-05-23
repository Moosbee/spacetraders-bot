import {
  Button,
  Flex,
  Popover,
  Progress,
  Space,
  Switch,
  Table,
  TableProps,
} from "antd";
import { useState } from "react";
import { Link } from "react-router-dom";
import { backendUrl } from "../data";
import PageTitle from "../features/PageTitle";
import RoleRenderer from "../features/RoleRenderer/RoleRenderer";
import Timer from "../features/Timer/Timer";
import { ShipNavFlightMode, ShipNavStatus, ShipRole } from "../models/api";
import RustShip, { SystemShipRoles } from "../models/ship";
import { useAppDispatch, useAppSelector } from "../redux/hooks";
import {
  resetShips,
  selectAllShipsArray,
  setShips,
} from "../redux/slices/shipSlice";
import { shallowEqual } from "../utils/utils";

type TableRowSelection<T extends object = object> =
  TableProps<T>["rowSelection"];

function Ships() {
  const [showCooldown, setShowCooldown] = useState(true);
  const [showCondition, setShowCondition] = useState(false);

  const [showSelection, setShowSelection] = useState<boolean>(false);

  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);

  const ships = useAppSelector(selectAllShipsArray);

  const dispatch = useAppDispatch();

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
      render: (symbol, record) => (
        <>
          <Link to={`/ships/${symbol}`}>{symbol}</Link> (
          {record.active ? "A" : "I"})
        </>
      ),
      sorter: (a, b) =>
        Number.parseInt(a.symbol.split("-")[1], 16) -
        Number.parseInt(b.symbol.split("-")[1], 16),
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
    },
    {
      title: "Role",
      dataIndex: "role",
      key: "role",
      filters: Object.values(SystemShipRoles).map((role) => ({
        text: role,
        value: role,
      })),
      render: (role: SystemShipRoles, record) => (
        <RoleRenderer role={role} status={record.status} />
      ),
      defaultFilteredValue: [
        "Construction",
        "Trader",
        "TempTrader",
        "Contract",
        "Mining",
        "Charter",
        "Manuel",
        "Transfer",
      ],
      onFilter: (value, record) => record.role === value,
      sorter: (a, b) => {
        const num = a.role.localeCompare(b.role);
        if (num === 0) {
          if (a.status.type === "Mining" && b.status.type === "Mining") {
            const data_a = a.status.data ?? "";
            const data_b = b.status.data ?? "";
            if (
              data_a.assignment.type === "Transporter" &&
              data_b.assignment.type === "Transporter"
            ) {
              return a.symbol.localeCompare(b.symbol);
            }
            if (
              (data_a.assignment.type === "Siphoner" &&
                data_b.assignment.type === "Siphoner") ||
              (data_a.assignment.type === "Extractor" &&
                data_b.assignment.type === "Extractor")
            ) {
              return a.nav.waypoint_symbol.localeCompare(b.nav.waypoint_symbol);
            }
            return data_a.assignment.type.localeCompare(data_b.assignment.type);
          }
          if (a.status.type === "Trader" && b.status.type === "Trader") {
            return a.symbol.localeCompare(b.symbol);
          }
          if (a.status.type === "Transfer" && b.status.type === "Transfer") {
            const num1 = a.status.data.role ?? "Transfer";
            const num2 = b.status.data.role ?? "Transfer";
            return num1.localeCompare(num2);
          }
        }
        return num;
      },
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
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
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
    },

    {
      title: "Current Waypoint",
      dataIndex: ["nav", "waypoint_symbol"],
      key: "current_waypoint",
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
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
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
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
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
      render: (value: ShipNavStatus, record) => (
        <span>
          {value}
          {value === "IN_TRANSIT" && (
            <>
              {" "}
              (<Timer time={record.nav.route.arrival} />)
              <br />
              <span>
                {record.nav.route.origin_system_symbol ==
                record.nav.route.destination_system_symbol
                  ? record.nav.route.origin_symbol.replace(
                      record.nav.route.origin_system_symbol + "-",
                      ""
                    )
                  : record.nav.route.origin_symbol}{" "}
                -{">"}{" "}
                {record.nav.route.origin_system_symbol ==
                record.nav.route.destination_system_symbol
                  ? record.nav.route.destination_symbol.replace(
                      record.nav.route.destination_system_symbol + "-",
                      ""
                    )
                  : record.nav.route.destination_symbol}
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
      sorter: (a, b) => {
        const num = a.nav.status.localeCompare(b.nav.status);
        if (num === 0) {
          if (a.nav.status === "IN_TRANSIT" && b.nav.status === "IN_TRANSIT") {
            const data_a = new Date(a.nav.route.arrival).getTime();
            const data_b = new Date(b.nav.route.arrival).getTime();
            return data_a - data_b;
          }
        }

        return num;
      },
    },

    {
      title: "Autopilot",
      key: "autopilot",
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
      align: "center",
      render: (_value, record) => (
        <>
          {record.nav.auto_pilot && (
            <span>
              {record.nav.auto_pilot.origin_system_symbol ==
              record.nav.auto_pilot.destination_system_symbol
                ? record.nav.auto_pilot.origin_symbol.replace(
                    record.nav.auto_pilot.origin_system_symbol + "-",
                    ""
                  )
                : record.nav.auto_pilot.origin_symbol}{" "}
              -{">"}{" "}
              {record.nav.auto_pilot.origin_system_symbol ===
              record.nav.auto_pilot.destination_system_symbol
                ? record.nav.auto_pilot.destination_symbol.replace(
                    record.nav.auto_pilot.destination_system_symbol + "-",
                    ""
                  )
                : record.nav.auto_pilot.destination_symbol}
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
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
      sorter: (a, b) => a.engine_speed - b.engine_speed,
      align: "right",
    },
    {
      title: "Cargo",
      dataIndex: ["cargo", "units"],
      key: "cargo_units",
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
      render: (value: number, record) => (
        <Popover
          content={
            <Flex vertical>
              {Object.entries(record.cargo.inventory).map((item) => (
                <Flex gap={6} justify="space-between" key={item[0]}>
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
      // https://github.com/ant-design/ant-design/issues/23763
      shouldCellUpdate: (record, prevRecord) =>
        !shallowEqual(record, prevRecord),
    },
    ...(showCondition
      ? [
          {
            title: "Conditions",
            key: "conditions",

            render: (_value: unknown, record: RustShip) => (
              <Space>
                <Progress
                  type="circle"
                  percent={record.conditions.engine.condition * 100}
                  format={(value) => (
                    <>
                      Engine: {value}%{" "}
                      {record.conditions.engine.integrity * 100}%
                    </>
                  )}
                  size={20}
                />
                <Progress
                  type="circle"
                  percent={record.conditions.frame.condition * 100}
                  size={20}
                  format={(value) => (
                    <>
                      Frame: {value}% {record.conditions.frame.integrity * 100}%
                    </>
                  )}
                />
                <Progress
                  type="circle"
                  percent={record.conditions.reactor.condition * 100}
                  size={20}
                  format={(value) => (
                    <>
                      Reactor: {value}%{" "}
                      {record.conditions.reactor.integrity * 100}%
                    </>
                  )}
                />
              </Space>
            ),
            // https://github.com/ant-design/ant-design/issues/23763
            shouldCellUpdate: (record: RustShip, prevRecord: RustShip) =>
              !shallowEqual(record, prevRecord),
            // align: "right",
          },
        ]
      : []),

    ...(showCooldown
      ? [
          {
            title: "Cooldown",
            dataIndex: "cooldown_expiration",
            key: "cooldown_expiration",
            render: (value: string | null) =>
              value && (
                <span
                  style={{
                    color:
                      new Date() < new Date(value) ? "currentColor" : "red",
                  }}
                >
                  <Timer time={value} />
                </span>
              ),
            // https://github.com/ant-design/ant-design/issues/23763
            shouldCellUpdate: (record: RustShip, prevRecord: RustShip) =>
              !shallowEqual(record, prevRecord),
          },
        ]
      : []),
  ];

  const onSelectChange = (newSelectedRowKeys: React.Key[]) => {
    console.log("selectedRowKeys changed: ", newSelectedRowKeys);
    setSelectedRowKeys(newSelectedRowKeys);
  };

  const rowSelection: TableRowSelection<RustShip> = {
    selectedRowKeys,
    onChange: onSelectChange,
    // selections: [
    //   Table.SELECTION_ALL,
    //   Table.SELECTION_INVERT,
    //   Table.SELECTION_NONE,
    //   {
    //     key: "Manuel",
    //     text: "Select Manuel Ships",
    //     onSelect: (changeableRowKeys) => {
    //       let newSelectedRowKeys = [];
    //       newSelectedRowKeys = changeableRowKeys.filter((shipKey) => {
    //         const ship = ships[shipKey as string];
    //         return ship.role !== "Manuel";
    //       });
    //       setSelectedRowKeys(newSelectedRowKeys);
    //     },
    //   },
    // ],
  };

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="All Ships" />
      <Space>
        <h2>All Ships</h2>
        <Button onClick={() => dispatch(resetShips())}>Reset</Button>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/ships`)
              .then((response) => response.json())
              .then((ships: Record<string, RustShip>) =>
                dispatch(setShips(ships))
              );
          }}
        >
          Refresh
        </Button>
        <Switch
          checked={showCooldown}
          onChange={(checked) => setShowCooldown(checked)}
        />
        Show Cooldown
        <Switch
          checked={showCondition}
          onChange={(checked) => setShowCondition(checked)}
        />
        Show Condition
        <Switch
          checked={showSelection}
          onChange={(checked) => {
            setShowSelection(checked);
            setSelectedRowKeys([]);
          }}
        />
        Show Selection
      </Space>
      <Table
        size="small"
        dataSource={ships}
        columns={columns}
        rowKey={(ship) => ship.symbol}
        rowSelection={showSelection ? rowSelection : undefined}
        pagination={{
          showSizeChanger: true,
          pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
          defaultPageSize: 100,
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}

        // virtual
      />
    </div>
  );
}

export default Ships;
