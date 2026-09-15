import { useQuery } from "@apollo/client/react";
import {
  Button,
  Card,
  Divider,
  Flex,
  Popover,
  Result,
  Space,
  Spin,
  Table,
  TableProps,
} from "antd";
import BoxPlot, { BoxPlotDatum } from "../features/BoxPlot/BoxPlot";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import {
  ActivityLevel,
  AllLatestShipyardShipsQuery,
  ShipEngineSymbol,
  ShipFrameSymbol,
  ShipReactorSymbol,
  ShipType,
  SupplyLevel,
} from "../gql/graphql";
import { GET_ALL_LATEST_SHIPYARD_SHIPS } from "../graphql/queries";

type ShipyardShip =
  AllLatestShipyardShipsQuery["shipyardShips"]["items"][number];

export default function ShipyardShips() {
  const { loading, error, data, dataState, refetch } = useQuery(
    GET_ALL_LATEST_SHIPYARD_SHIPS,
  );

  const items = data?.shipyardShips.items ?? [];

  if (error) {
    return (
      <div style={{ padding: "24px 24px" }}>
        <PageTitle title="Shipyard Ships" />
        <Result
          status="error"
          title="Error"
          subTitle={`Error: ${error.message}`}
          extra={[
            <Button key="retry" type="primary" onClick={() => refetch()}>
              Try Again
            </Button>,
          ]}
        />
      </div>
    );
  }

  const boxData = (): BoxPlotDatum[] => {
    const groups = new Map<string, number[]>();
    items.forEach((item) => {
      const list = groups.get(item.shipType) ?? [];
      list.push(item.purchasePrice);
      groups.set(item.shipType, list);
    });
    return [...groups.entries()]
      .map(([label, values]) => ({ label, values }))
      .sort((a, b) => a.label.localeCompare(b.label));
  };

  const columns: TableProps<ShipyardShip>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "Waypoint",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (waypoint) => (
        <WaypointLink waypoint={waypoint}>{waypoint}</WaypointLink>
      ),
      sorter: (a, b) => a.waypointSymbol.localeCompare(b.waypointSymbol),
    },
    {
      title: "Ship Type",
      dataIndex: "shipType",
      key: "shipType",
      filters: Object.values(ShipType).map((shipType) => ({
        text: shipType,
        value: shipType,
      })),
      onFilter: (value, record) => record.shipType === value,
      sorter: (a, b) => a.shipType.localeCompare(b.shipType),
    },
    {
      title: "Name",
      dataIndex: "name",
      key: "name",
      sorter: (a, b) => a.name.localeCompare(b.name),
    },
    {
      title: "Supply",
      dataIndex: "supply",
      key: "supply",
      filters: Object.values(SupplyLevel).map((supply) => ({
        text: supply,
        value: supply,
      })),
      onFilter: (value, record) => record.supply === value,
      sorter: (a, b) => a.supply.localeCompare(b.supply),
    },
    {
      title: "Activity",
      dataIndex: "activity",
      key: "activity",
      render: (activity) => activity ?? "N/A",
      filters: Object.values(ActivityLevel).map((activity) => ({
        text: activity,
        value: activity,
      })),
      onFilter: (value, record) => record.activity === value,
      sorter: (a, b) => (a.activity ?? "").localeCompare(b.activity ?? ""),
    },
    {
      title: "Purchase Price",
      dataIndex: "purchasePrice",
      key: "purchasePrice",
      align: "right",
      render: (price) => <MoneyDisplay amount={price} />,
      sorter: (a, b) => a.purchasePrice - b.purchasePrice,
    },
    {
      title: "Frame",
      key: "frame",
      render: (_, record) =>
        `${record.frameType}${
          record.frameQuality != null ? ` (${record.frameQuality})` : ""
        }`,
      filters: Object.values(ShipFrameSymbol).map((frame) => ({
        text: frame,
        value: frame,
      })),
      onFilter: (value, record) => record.frameType === value,
      sorter: (a, b) => a.frameType.localeCompare(b.frameType),
    },
    {
      title: "Reactor",
      key: "reactor",
      render: (_, record) =>
        `${record.reactorType}${
          record.reactorQuality != null ? ` (${record.reactorQuality})` : ""
        }`,
      filters: Object.values(ShipReactorSymbol).map((reactor) => ({
        text: reactor,
        value: reactor,
      })),
      onFilter: (value, record) => record.reactorType === value,
      sorter: (a, b) => a.reactorType.localeCompare(b.reactorType),
    },
    {
      title: "Engine",
      key: "engine",
      render: (_, record) =>
        `${record.engineType}${
          record.engineQuality != null ? ` (${record.engineQuality})` : ""
        }`,
      filters: Object.values(ShipEngineSymbol).map((engine) => ({
        text: engine,
        value: engine,
      })),
      onFilter: (value, record) => record.engineType === value,
      sorter: (a, b) => a.engineType.localeCompare(b.engineType),
    },
    {
      title: "Modules",
      key: "modules",
      align: "end",
      render: (_, record) => (
        <Popover
          title={
            <Flex vertical>
              {record.modules.map((module) => (
                <span key={module}>{module}</span>
              ))}
            </Flex>
          }
          className="flex justify-between items-center"
        >
          <span>
            {record.modules
              .map((module) =>
                module
                  .split("_")
                  .map((part) =>
                    part === "I"
                      ? "1"
                      : part === "II"
                        ? "2"
                        : part === "III"
                          ? "3"
                          : part[0],
                  )
                  .join(""),
              )
              .join(", ")}
          </span>
          <span>{record.modules.length}</span>
        </Popover>
      ),
    },
    {
      title: "Mounts",
      key: "mounts",
      align: "end",
      render: (_, record) => (
        <Popover
          title={
            <Flex vertical>
              {record.mounts.map((mount) => (
                <span key={mount}>{mount}</span>
              ))}
            </Flex>
          }
          className="flex justify-between items-center"
        >
          <span>
            {record.mounts
              .map((mount) =>
                mount
                  .split("_")
                  .map((part) =>
                    part === "I"
                      ? "1"
                      : part === "II"
                        ? "2"
                        : part === "III"
                          ? "3"
                          : part[0],
                  )
                  .join(""),
              )
              .join(", ")}
          </span>
          <span>{record.mounts.length}</span>
        </Popover>
      ),
    },
    {
      title: "Crew Requirement",
      dataIndex: "crewRequirement",
      key: "crewRequirement",
      align: "right",
      sorter: (a, b) => a.crewRequirement - b.crewRequirement,
    },
    {
      title: "Crew Capacity",
      dataIndex: "crewCapacity",
      key: "crewCapacity",
      align: "right",
      sorter: (a, b) => a.crewCapacity - b.crewCapacity,
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Shipyard Ships" />
      <Space>
        <h1>Shipyard Ships {items.length}</h1>
        <Button onClick={() => refetch()}>Refresh</Button>
        <Spin spinning={loading || dataState !== "complete"} />
      </Space>
      <Card variant="borderless" title="Purchase Price by Ship Type">
        <BoxPlot data={boxData()} />
      </Card>
      <Divider />
      <Table<ShipyardShip>
        dataSource={items}
        rowKey="id"
        columns={columns}
        pagination={{
          showSizeChanger: true,
          pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
          defaultPageSize: 10,
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
    </div>
  );
}
