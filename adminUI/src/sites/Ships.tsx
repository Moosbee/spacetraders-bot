import { useQuery } from "@apollo/client/react";
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
import PageTitle from "../features/PageTitle";
import RoleRenderer from "../features/RoleRenderer/RoleRenderer";
import Timer from "../features/Timer/Timer";
import {
  GetAllShipsQuery,
  ShipNavFlightMode,
  ShipNavStatus,
  ShipRole,
} from "../gql/graphql";
import { GET_ALL_SHIPS } from "../graphql/queries";

type ShipData = GetAllShipsQuery["ships"][number];

type TableRowSelection<T extends object = object> =
  TableProps<T>["rowSelection"];

function Ships() {
  const [showCooldown, setShowCooldown] = useState(true);
  const [showCondition, setShowCondition] = useState(false);
  const [showSelection, setShowSelection] = useState<boolean>(false);
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);

  const { loading, data, refetch } = useQuery(GET_ALL_SHIPS);
  const ships = data?.ships ?? [];

  const columns: TableProps<ShipData>["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      defaultSortOrder: "ascend",
      render: (symbol: string) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) =>
        Number.parseInt(a.symbol.split("-")[1], 16) -
        Number.parseInt(b.symbol.split("-")[1], 16),
    },
    {
      title: "Status",
      key: "status",
      render: (_role, record) => <RoleRenderer status={record.status} />,
      filters: [
        {
          text: "Charting",
          value: "ChartingStatus",
        },
        {
          text: "Construction",
          value: "ConstructionStatus",
        },
        {
          text: "Contract",
          value: "ContractStatus",
        },
        {
          text: "Manuel",
          value: "ManuelStatus",
        },
        {
          text: "Mining",
          value: "MiningStatus",
        },
        {
          text: "Scraper",
          value: "ScraperStatus",
        },
        {
          text: "Trader",
          value: "TraderStatus",
        },
        {
          text: "Transfer",
          value: "TransferStatus",
        },
      ],
      onFilter: (value, record) => record.status.status.__typename === value,
      sorter: (a, b) => {
        const num = (a.status.status.__typename ?? "").localeCompare(
          b.status.status.__typename ?? "",
        );
        if (num === 0) {
          const aType = a.status.status.__typename;
          const bType = b.status.status.__typename;
          if (aType === "MiningStatus" && bType === "MiningStatus") {
            const aAssign = a.status.status.assignment.__typename ?? "";
            const bAssign = b.status.status.assignment.__typename ?? "";
            if (
              aAssign === "TransporterAssignment" &&
              bAssign === "TransporterAssignment"
            ) {
              return a.symbol.localeCompare(b.symbol);
            }
            if (
              (aAssign === "SiphonerAssignment" &&
                bAssign === "SiphonerAssignment") ||
              (aAssign === "ExtractorAssignment" &&
                bAssign === "ExtractorAssignment")
            ) {
              return a.nav.waypointSymbol.localeCompare(b.nav.waypointSymbol);
            }
            return aAssign.localeCompare(bAssign);
          }
          if (aType === "TraderStatus" && bType === "TraderStatus") {
            return a.symbol.localeCompare(b.symbol);
          }
          if (aType === "TransferStatus" && bType === "TransferStatus") {
            return a.status.status.assignmentId - b.status.status.assignmentId;
          }
        }
        return num;
      },
    },
    {
      title: "Fleet",
      key: "fleetId",
      filters: [
        ...[
          ...new Set(
            ships
              .map((ship) => ship.status?.fleetId)
              .filter((id) => !!id)
              .map((id) => id as unknown as number),
          ),
        ]
          .toSorted((a, b) => a - b)
          .map((id) => ({ text: id, value: id })),
      ],
      onFilter: (value, record) => record.status?.fleetId === value,
      render: (_role, record) => (
        <span>
          {record.status?.fleetId} ({record.status.assignmentId})
        </span>
      ),
      sorter: (a, b) => (a.status?.fleetId || 0) - (b.status?.fleetId || 0),
    },
    {
      title: "Registration Role",
      dataIndex: "registrationRole",
      key: "registrationRole",
      filters: Object.values(ShipRole).map((role) => ({
        text: role,
        value: role,
      })),
      onFilter: (value, record) => record.registrationRole === value,
      sorter: (a, b) => a.registrationRole.localeCompare(b.registrationRole),
    },

    {
      title: "Current Waypoint",
      dataIndex: ["nav", "waypointSymbol"],
      key: "current_waypoint",
      sorter: (a, b) =>
        a.nav.waypointSymbol.localeCompare(b.nav.waypointSymbol),
      render: (value: string, record) => (
        <span>
          <Link to={`/system/${record.nav.systemSymbol}`}>
            {record.nav.systemSymbol}
          </Link>
          <Link to={`/system/${record.nav.systemSymbol}/${value}`}>
            {record.nav.waypointSymbol.replace(record.nav.systemSymbol, "")}
          </Link>
        </span>
      ),
    },
    {
      title: "Flight Mode",
      dataIndex: ["nav", "flightMode"],
      key: "flightMode",
      filters: Object.values(ShipNavFlightMode).map((role) => ({
        text: role,
        value: role,
      })),
      onFilter: (value, record) => record.nav.flightMode === value,
      sorter: (a, b) => a.nav.flightMode.localeCompare(b.nav.flightMode),
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
                {record.nav.route.originSystemSymbol ===
                record.nav.route.destinationSystemSymbol
                  ? record.nav.route.originSymbol.replace(
                      record.nav.route.originSystemSymbol + "-",
                      "",
                    )
                  : record.nav.route.originSymbol}{" "}
                -{">"}{" "}
                {record.nav.route.originSystemSymbol ===
                record.nav.route.destinationSystemSymbol
                  ? record.nav.route.destinationSymbol.replace(
                      record.nav.route.destinationSystemSymbol + "-",
                      "",
                    )
                  : record.nav.route.destinationSymbol}
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
      align: "center",
      render: (_value, record) => (
        <>
          {record.nav.autoPilot && (
            <span>
              {record.nav.autoPilot.originSystemSymbol ==
              record.nav.autoPilot.destinationSystemSymbol
                ? record.nav.autoPilot.originSymbol.replace(
                    record.nav.autoPilot.originSystemSymbol + "-",
                    "",
                  )
                : record.nav.autoPilot.originSymbol}{" "}
              -{">"}{" "}
              {record.nav.autoPilot.originSystemSymbol ===
              record.nav.autoPilot.destinationSystemSymbol
                ? record.nav.autoPilot.destinationSymbol.replace(
                    record.nav.autoPilot.destinationSystemSymbol + "-",
                    "",
                  )
                : record.nav.autoPilot.destinationSymbol}
              <br />(<Timer time={record.nav.autoPilot.arrival} />)
            </span>
          )}
        </>
      ),
    },
    {
      title: "Engine Speed",
      dataIndex: "engineSpeed",
      key: "engineSpeed",
      sorter: (a, b) => a.engineSpeed - b.engineSpeed,
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
              {record.cargo.inventory.map((item) => (
                <Flex gap={6} justify="space-between" key={item.symbol}>
                  <span>{item.symbol}</span>
                  <span>{item.units}</span>
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
    ...(showCondition
      ? [
          {
            title: "Conditions",
            key: "conditions",
            render: (_value: unknown, record: ShipData) => (
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
          },
        ]
      : []),

    ...(showCooldown
      ? [
          {
            title: "Cooldown",
            dataIndex: "cooldownExpiration",
            key: "cooldownExpiration",
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
          },
        ]
      : []),
  ];

  const onSelectChange = (newSelectedRowKeys: React.Key[]) => {
    console.log("selectedRowKeys changed: ", newSelectedRowKeys);
    setSelectedRowKeys(newSelectedRowKeys);
  };

  const rowSelection: TableRowSelection<ShipData> = {
    selectedRowKeys,
    onChange: onSelectChange,
  };

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="All Ships" />
      <Space>
        <h2>All Ships</h2>
        <Button onClick={() => refetch()} loading={loading}>
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
      />
    </div>
  );
}

export default Ships;
