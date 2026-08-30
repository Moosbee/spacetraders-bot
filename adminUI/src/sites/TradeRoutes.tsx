import { useQuery } from "@apollo/client/react";
import { Button, Space, Table, TableProps } from "antd";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import {
  GetAllTradeRoutesQuery,
  ShipmentStatus,
  TradeMode,
  TradeSymbol,
} from "../gql/graphql";
import { GET_ALL_TRADE_ROUTES } from "../graphql/queries";

type TradeRoute = NonNullable<
  GetAllTradeRoutesQuery["tradeRoutes"]
>["items"][number];

const predictedProfit = (route: TradeRoute) =>
  (route.sellMarketTradeGood?.sellPrice ?? 0) * route.tradeVolume -
  (route.purchaseMarketTradeGood?.purchasePrice ?? 0) * route.tradeVolume -
  (route.estimatedFuel ?? 0);

const actualProfit = (route: TradeRoute) =>
  (route.marketTransactionSummary.allIncome ?? 0) -
  (route.marketTransactionSummary.allExpenses ?? 0);

function TradeRoutes() {
  const { loading, error, data, refetch } = useQuery(GET_ALL_TRADE_ROUTES);

  const tradeRoutes = data?.tradeRoutes.items ?? [];

  const columns: TableProps<TradeRoute>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "Created At",
      dataIndex: "createdAt",
      key: "createdAt",
      render: (date: string) => new Date(date).toLocaleString(),
      sorter: (a, b) =>
        new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
    },
    {
      title: "Trade Symbol",
      dataIndex: "symbol",
      key: "symbol",
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
      filters: Object.values(TradeSymbol).map((symbol) => ({
        text: symbol,
        value: symbol,
      })),
      filterSearch: true,
      onFilter: (value, record) => record.symbol === value,
      render: (value) => <Link to={`/supplyChain/${value}`}>{value}</Link>,
    },
    {
      title: "Ship",
      dataIndex: "shipSymbol",
      key: "shipSymbol",
      render: (symbol: string) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) => a.shipSymbol.localeCompare(b.shipSymbol),
    },
    {
      title: "Purchase Waypoint",
      dataIndex: "PurchaseWaypointSymbol",
      key: "PurchaseWaypointSymbol",
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
      sorter: (a, b) =>
        a.PurchaseWaypointSymbol.localeCompare(b.PurchaseWaypointSymbol),
    },
    {
      title: "Sell Waypoint",
      dataIndex: "SellWaypointSymbol",
      key: "SellWaypointSymbol",
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
      sorter: (a, b) =>
        a.SellWaypointSymbol.localeCompare(b.SellWaypointSymbol),
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
      sorter: (a, b) => a.status.localeCompare(b.status),
      filters: Object.values(ShipmentStatus).map((status) => ({
        text: status,
        value: status,
      })),
      onFilter: (value, record) => record.status === value,
    },
    {
      title: "Trade Mode",
      dataIndex: "tradeMode",
      key: "tradeMode",
      sorter: (a, b) => a.tradeMode.localeCompare(b.tradeMode),
      filters: Object.values(TradeMode).map((mode) => ({
        text: mode,
        value: mode,
      })),
      onFilter: (value, record) => record.tradeMode === value,
    },
    {
      title: "Volume",
      dataIndex: "tradeVolume",
      key: "tradeVolume",
      align: "right",
      sorter: (a, b) => a.tradeVolume - b.tradeVolume,
    },
    {
      title: "Reserved Fund",
      dataIndex: "reservedFund",
      key: "reservedFund",
      render: (value: number | null | undefined) => (
        <MoneyDisplay amount={value ?? 0} />
      ),
      align: "right",
      sorter: (a, b) => (a.reservedFund ?? 0) - (b.reservedFund ?? 0),
    },
    {
      title: "Purchase Cost",
      key: "purchaseCost",
      render: (_, record) => (
        <MoneyDisplay
          amount={
            (record.purchaseMarketTradeGood?.purchasePrice ?? 0) *
            record.tradeVolume
          }
        />
      ),
      align: "right",
      sorter: (a, b) =>
        (a.purchaseMarketTradeGood?.purchasePrice ?? 0) * a.tradeVolume -
        (b.purchaseMarketTradeGood?.purchasePrice ?? 0) * b.tradeVolume,
    },
    {
      title: "Sell Revenue",
      key: "sellRevenue",
      render: (_, record) => (
        <MoneyDisplay
          amount={
            (record.sellMarketTradeGood?.sellPrice ?? 0) * record.tradeVolume
          }
        />
      ),
      align: "right",
      sorter: (a, b) =>
        (a.sellMarketTradeGood?.sellPrice ?? 0) * a.tradeVolume -
        (b.sellMarketTradeGood?.sellPrice ?? 0) * b.tradeVolume,
    },
    {
      title: "Est. Fuel",
      dataIndex: "estimatedFuel",
      key: "estimatedFuel",
      render: (value: number | null | undefined) => (
        <MoneyDisplay amount={value ?? 0} />
      ),
      align: "right",
      sorter: (a, b) => (a.estimatedFuel ?? 0) - (b.estimatedFuel ?? 0),
    },
    {
      title: "Predicted Profit",
      key: "predictedProfit",
      render: (_, record) => <MoneyDisplay amount={predictedProfit(record)} />,
      align: "right",
      sorter: (a, b) => predictedProfit(a) - predictedProfit(b),
    },
    {
      title: "Expenses",
      key: "expenses",
      render: (_, record) => (
        <MoneyDisplay
          amount={record.marketTransactionSummary?.allExpenses ?? 0}
        />
      ),
      align: "right",
      sorter: (a, b) =>
        (a.marketTransactionSummary?.allExpenses ?? 0) -
        (b.marketTransactionSummary?.allExpenses ?? 0),
    },
    {
      title: "Income",
      key: "income",
      render: (_, record) => (
        <MoneyDisplay
          amount={record.marketTransactionSummary?.allIncome ?? 0}
        />
      ),
      align: "right",
      sorter: (a, b) =>
        (a.marketTransactionSummary?.allIncome ?? 0) -
        (b.marketTransactionSummary?.allIncome ?? 0),
    },
    {
      title: "Profit",
      key: "profit",
      render: (_, record) => {
        const profit = actualProfit(record);
        return (
          <MoneyDisplay
            amount={profit}
            style={{ color: profit < 0 ? "red" : "currentColor" }}
          />
        );
      },
      align: "right",
      sorter: (a, b) => actualProfit(a) - actualProfit(b),
    },
    {
      title: "Delta",
      key: "delta",
      render: (_, record) => (
        <MoneyDisplay amount={actualProfit(record) - predictedProfit(record)} />
      ),
      align: "right",
      sorter: (a, b) =>
        actualProfit(a) -
        predictedProfit(a) -
        (actualProfit(b) - predictedProfit(b)),
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title={`TradeRoutes ${tradeRoutes.length}`} />
      <Space>
        <h1>TradeRoutes {tradeRoutes.length}</h1>
        <Button loading={loading} onClick={() => refetch()}>
          Refresh
        </Button>
      </Space>
      {error && <p>Error: {error.message}</p>}
      <Table
        dataSource={tradeRoutes}
        columns={columns}
        rowKey="id"
        loading={loading}
        scroll={{ x: "max-content" }}
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

export default TradeRoutes;
