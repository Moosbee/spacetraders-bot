import { useQuery } from "@apollo/client/react";
import { Button, Divider, Progress, Space, Table, TableProps } from "antd";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { GetConstructionQuery } from "../gql/graphql";
import { GET_CONSTRUCTION } from "../graphql/queries";

type GQLConstructionMaterial =
  GetConstructionQuery["constructionMaterials"]["items"][number];
type GQLConstructionShipment =
  GetConstructionQuery["constructionShipments"]["items"][number];

function Construction() {
  const { loading, error, data, dataState, refetch } =
    useQuery(GET_CONSTRUCTION);

  if (dataState !== "complete") return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  const materials = data.constructionMaterials.items;
  const shipments = data.constructionShipments.items;
  const runningShipments = data.constructionManager.runningShipments;

  const materialColumns: TableProps<GQLConstructionMaterial>["columns"] = [
    {
      title: "id",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "waypoint_symbol",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      sorter: (a, b) => a.waypointSymbol.localeCompare(b.waypointSymbol),
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
    },
    {
      title: "trade_symbol",
      dataIndex: "tradeSymbol",
      key: "tradeSymbol",
      sorter: (a, b) => a.tradeSymbol.localeCompare(b.tradeSymbol),
    },
    {
      title: "required",
      dataIndex: "required",
      key: "required",
      sorter: (a, b) => a.required - b.required,
      align: "right",
    },
    {
      title: "fulfilled",
      dataIndex: "fulfilled",
      key: "fulfilled",
      sorter: (a, b) => a.fulfilled - b.fulfilled,
      align: "right",
    },
    {
      title: "created_at",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
    },
    {
      title: "updated_at",
      dataIndex: "updatedAt",
      key: "updatedAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
    },
    {
      title: "Percent",
      dataIndex: "",
      key: "percent",
      render: (_, record) => (
        <Progress
          percent={
            record.required ? (record.fulfilled / record.required) * 100 : 0
          }
          size={"small"}
        />
      ),
    },
    {
      title: "expenses",
      dataIndex: ["marketTransactionSummary", "allExpenses"],
      key: "expenses",
      render: (value) => <MoneyDisplay amount={value || 0} />,
      sorter: (a, b) =>
        (a.marketTransactionSummary.allExpenses || 0) -
        (b.marketTransactionSummary.allExpenses || 0),
      align: "right",
    },
    {
      title: "income",
      dataIndex: ["marketTransactionSummary", "allIncome"],
      key: "income",
      render: (value) => <MoneyDisplay amount={value || 0} />,
      sorter: (a, b) =>
        (a.marketTransactionSummary.allIncome || 0) -
        (b.marketTransactionSummary.allIncome || 0),
      align: "right",
    },
  ];

  const shipmentColumns: TableProps<GQLConstructionShipment>["columns"] = [
    {
      title: "id",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "material_id",
      dataIndex: "materialId",
      key: "materialId",
      sorter: (a, b) => a.materialId - b.materialId,
    },
    {
      title: "construction_site_waypoint",
      dataIndex: "constructionSiteWaypoint",
      key: "constructionSiteWaypoint",
      sorter: (a, b) =>
        a.constructionSiteWaypoint.localeCompare(b.constructionSiteWaypoint),
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
    },
    {
      title: "ship_symbol",
      dataIndex: "shipSymbol",
      key: "shipSymbol",
      sorter: (a, b) => a.shipSymbol.localeCompare(b.shipSymbol),
      render: (symbol: string) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
      filters: [...new Set(shipments.map((t) => t.shipSymbol))].map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.shipSymbol === value,
    },
    {
      title: "trade_symbol",
      dataIndex: "tradeSymbol",
      key: "tradeSymbol",
      sorter: (a, b) => a.tradeSymbol.localeCompare(b.tradeSymbol),
      filters: [...new Set(shipments.map((t) => t.tradeSymbol))].map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.tradeSymbol === value,
    },
    {
      title: "units",
      dataIndex: "units",
      key: "units",
      sorter: (a, b) => a.units - b.units,
      align: "right",
    },
    {
      title: "purchase_waypoint",
      dataIndex: "purchaseSiteWaypoint",
      key: "purchaseSiteWaypoint",
      sorter: (a, b) =>
        a.purchaseSiteWaypoint.localeCompare(b.purchaseSiteWaypoint),
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
      render: (value) => value,
      sorter: (a, b) => (a.status === b.status ? 0 : a.status ? -1 : 1),
      filters: [
        { text: "Delivered", value: "DELIVERED" },
        { text: "InTransit", value: "IN_TRANSIT" },
        { text: "Failed", value: "FAILED" },
      ],
      onFilter: (value, record) => record.status === value,
    },
    {
      title: "created_at",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
    },
    {
      title: "updated_at",
      dataIndex: "updatedAt",
      key: "updatedAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
    },
    {
      title: "expenses",
      dataIndex: ["marketTransactionSummary", "allExpenses"],
      key: "expenses",
      render: (value) => <MoneyDisplay amount={value || 0} />,
      sorter: (a, b) =>
        (a.marketTransactionSummary.allExpenses || 0) -
        (b.marketTransactionSummary.allExpenses || 0),
      align: "right",
    },
    {
      title: "income",
      dataIndex: ["marketTransactionSummary", "allIncome"],
      key: "income",
      render: (value) => <MoneyDisplay amount={value || 0} />,
      sorter: (a, b) =>
        (a.marketTransactionSummary.allIncome || 0) -
        (b.marketTransactionSummary.allIncome || 0),
      align: "right",
    },
  ];

  const pagination = {
    showSizeChanger: true,
    pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
    defaultPageSize: 100,
    showTotal: (total: number, range: [number, number]) =>
      `${range[0]}-${range[1]} of ${total}`,
  };

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Construction" />
      <Space>
        <h1>Construction</h1>
        <Button onClick={() => refetch()}>Refresh</Button>
      </Space>
      <Divider size="small" />
      <Table
        title={() => "Construction Materials"}
        dataSource={materials || []}
        columns={materialColumns}
        rowKey="id"
        loading={loading}
        pagination={pagination}
      />
      <Divider size="small" />
      <Table
        title={() => "Running Construction Shipments"}
        dataSource={runningShipments || []}
        columns={shipmentColumns}
        rowKey="id"
        loading={loading}
        pagination={pagination}
      />
      <Divider size="small" />
      <Table
        title={() => "All Construction Shipments"}
        dataSource={shipments || []}
        columns={shipmentColumns}
        rowKey="id"
        loading={loading}
        pagination={pagination}
      />
    </div>
  );
}

export default Construction;
