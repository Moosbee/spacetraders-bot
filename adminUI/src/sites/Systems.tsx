import { useQuery } from "@apollo/client/react";
import { Button, Flex, Popover, Space, Spin, Table, TableProps } from "antd";
import { Link } from "react-router-dom";
import PageTitle from "../features/PageTitle";
import { WaypointType } from "../gql/graphql";
import { GET_ALL_SYSTEMS } from "../graphql/queries";
import { SystemType } from "../models/api";

function Systems() {
  const { loading, error, data, dataState, refetch } =
    useQuery(GET_ALL_SYSTEMS);

  if (dataState != "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  type SystemInfo = (typeof data)["systems"][number];

  const columns: TableProps<SystemInfo>["columns"] = [
    {
      title: "Symbol",
      dataIndex: "symbol",
      key: "symbol",
      render: (symbol) => <Link to={`/system/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Constellation",
      dataIndex: "constellation",
      key: "constellation",
      sorter: (a, b) =>
        (a.constellation || "").localeCompare(b.constellation || ""),
    },
    {
      title: "Sector Symbol",
      dataIndex: "sectorSymbol",
      key: "sectorSymbol",
      sorter: (a, b) => a.sectorSymbol.localeCompare(b.sectorSymbol),
      filters: [...new Set(data.systems?.map((s) => s.sectorSymbol) || [])].map(
        (s) => ({
          text: s,
          value: s,
        })
      ),
      onFilter: (value, record) => record.sectorSymbol === value,
    },
    {
      title: "System Type",
      dataIndex: "systemType",
      key: "systemType",
      sorter: (a, b) => a.systemType.localeCompare(b.systemType),
      filters: Object.values(SystemType).map((s) => ({
        text: s,
        value: s,
      })),
      onFilter: (value, record) => record.systemType === value,
    },
    {
      title: "X",
      dataIndex: "x",
      key: "x",
      sorter: (a, b) => a.x - b.x,
    },
    {
      title: "Y",
      dataIndex: "y",
      key: "y",
      sorter: (a, b) => a.y - b.y,
    },
    {
      title: "Waypoints",
      render: (_, system) => (
        <Popover
          content={
            <Flex vertical>
              {Object.entries(
                system.waypoints
                  .map((wp) => wp.waypointType)
                  .reduce((curr, wp) => {
                    curr[wp] = (curr[wp] || 0) + 1;
                    return curr;
                  }, {} as Record<WaypointType, number>)
              ).map((item) => (
                <Flex gap={6} justify="space-between" key={item[0]}>
                  <span>{item[0]}</span>
                  <span>{item[1]}</span>
                </Flex>
              ))}
            </Flex>
          }
        >
          {system.waypoints.length}
        </Popover>
      ),
      key: "waypoints",
      sorter: (a, b) => (a.waypoints.length || 0) - (b.waypoints.length || 0),
    },
    {
      title: "Marketplaces",
      render: (_, system) => (
        <Popover
          content={
            <Flex vertical>
              {system.waypoints
                .filter((wp) => wp.hasMarketplace)
                .map((item) => (
                  <Flex gap={6} justify="space-between" key={item.symbol}>
                    <span>{item.symbol}</span>
                  </Flex>
                ))}
            </Flex>
          }
        >
          {system.waypoints.filter((wp) => wp.hasMarketplace).length}
        </Popover>
      ),
      key: "marketplaces",
      sorter: (a, b) =>
        (a.waypoints.filter((wp) => wp.hasMarketplace).length || 0) -
        (b.waypoints.filter((wp) => wp.hasMarketplace).length || 0),
    },
    {
      title: "Shipyards",
      render: (_, system) => (
        <Popover
          content={
            <Flex vertical>
              {system.waypoints
                .filter((wp) => wp.hasShipyard)
                .map((item) => (
                  <Flex gap={6} justify="space-between" key={item.symbol}>
                    <span>{item.symbol}</span>
                  </Flex>
                ))}
            </Flex>
          }
        >
          {system.waypoints.filter((wp) => wp.hasShipyard).length}
        </Popover>
      ),
      key: "shipyards",
      sorter: (a, b) =>
        (a.waypoints.filter((wp) => wp.hasShipyard).length || 0) -
        (b.waypoints.filter((wp) => wp.hasShipyard).length || 0),
    },
    {
      title: "Fleets",
      key: "fleets",
      sorter: (a, b) => (a.fleets.length || 0) - (b.fleets.length || 0),
      render: (_, system) => (
        <Popover
          content={
            <Flex vertical>
              {system.fleets.map((item) => (
                <Flex gap={6} justify="space-between" key={item.id}>
                  <span>{item.id}</span>
                  <span>{item.fleetType}</span>
                  <span>({item.active ? "A" : "I"})</span>
                </Flex>
              ))}
            </Flex>
          }
        >
          {system.fleets.length}
        </Popover>
      ),
    },
    {
      title: "Ships",
      key: "ships",
      sorter: (a, b) => (a.ships.length || 0) - (b.ships.length || 0),
      render: (_, system) => (
        <Popover
          content={
            <Flex vertical>
              {system.ships.map((item) => (
                <Flex gap={6} justify="space-around" key={item.symbol}>
                  <span>{item.symbol}</span>
                </Flex>
              ))}
            </Flex>
          }
        >
          {system.ships.length}
        </Popover>
      ),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Systems" />
      <Spin spinning={loading}>
        <Space>
          <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
            Systems {data.systems.length}
          </h1>
          <Button
            onClick={() => {
              refetch();
            }}
          >
            Refresh
          </Button>
        </Space>
        <Table
          dataSource={data.systems || []}
          columns={columns}
          rowKey="id"
          pagination={{
            showSizeChanger: true,
            pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
            defaultPageSize: 100,
            showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
          }}
        />
      </Spin>
    </div>
  );
}

export default Systems;
