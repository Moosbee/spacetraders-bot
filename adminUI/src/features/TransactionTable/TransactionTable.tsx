import { Table, TableProps } from "antd";
import { Link } from "react-router-dom";
import { MarketTransactionTypeEnum, TradeSymbol } from "../../models/api";
import { Transaction } from "../../models/Transaction";
import MoneyDisplay from "../MonyDisplay";
import WaypointLink from "../WaypointLink";

function TransactionTable({
  transactions,
  reasons = { contract: true, trade_route: true, mining: true },
}: {
  transactions: Transaction[];
  reasons?: { contract: boolean; trade_route: boolean; mining: boolean };
}) {
  const columns: TableProps<Transaction>["columns"] = [
    {
      title: "Waypoint",
      dataIndex: "waypoint_symbol",
      key: "waypoint_symbol",
      render: (symbol: string) => (
        <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
      ),
      sorter: (a, b) => a.waypoint_symbol.localeCompare(b.waypoint_symbol),

      filters: [...new Set(transactions.map((t) => t.waypoint_symbol))].map(
        (t) => ({
          text: t,
          value: t,
        })
      ),
      onFilter: (value, record) => record.waypoint_symbol === value,
    },
    {
      title: "Ship",
      dataIndex: "ship_symbol",
      key: "ship_symbol",
      render: (symbol: string) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) => a.ship_symbol.localeCompare(b.ship_symbol),
    },
    {
      title: "Trade Symbol",
      dataIndex: "trade_symbol",
      key: "trade_symbol",
      sorter: (a, b) => a.trade_symbol.localeCompare(b.trade_symbol),
      filters: Object.values(TradeSymbol)
        .sort((a, b) => a.localeCompare(b))
        .map((type) => ({
          text: type,
          value: type,
        })),
      onFilter: (value, record) => record.trade_symbol === value,
    },
    {
      title: "Transaction Type",
      dataIndex: "type",
      key: "type",
      sorter: (a, b) => a.type.localeCompare(b.type),
      filters: Object.values(MarketTransactionTypeEnum).map((type) => ({
        text: type,
        value: type,
      })),
      onFilter: (value, record) => record.type === value,
    },
    {
      title: "Units",
      dataIndex: "units",
      key: "units",
      align: "right",
      sorter: (a, b) => a.units - b.units,
    },
    {
      title: "Price Per Unit",
      dataIndex: "price_per_unit",
      key: "price_per_unit",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.price_per_unit - b.price_per_unit,
    },
    {
      title: "Total Price",
      dataIndex: "total_price",
      key: "total_price",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.total_price - b.total_price,
    },
    {
      title: "Timestamp",
      dataIndex: "timestamp",
      key: "timestamp",
      render: (value) => new Date(value).toLocaleString(),
      align: "right",
      sorter: (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime(),
      defaultSortOrder: "descend",
    },

    ...(reasons?.contract
      ? [
          {
            title: "Contract",
            dataIndex: "contract",
            key: "contract",
            render: (text: string | null) => text || "N/A", // Display "N/A" if null
            sorter: (a: Transaction, b: Transaction) =>
              (a.contract ?? "").localeCompare(b.contract ?? ""),
            filters: [
              {
                text: "Yes",
                value: "Yes",
              },
              {
                text: "No",
                value: "No",
              },
            ],
            onFilter: (value: boolean | React.Key, record: Transaction) =>
              (value === "No" && !record.contract) ||
              (value === "Yes" && !!record.contract),
          },
        ]
      : []),
    ...(reasons?.trade_route
      ? [
          {
            title: "Trade Route",
            dataIndex: "trade_route",
            key: "trade_route",
            render: (value: number | null) => (value !== null ? value : "N/A"), // Display "N/A" if null
            sorter: (a: Transaction, b: Transaction) =>
              (a.trade_route ?? 0) - (b.trade_route ?? 0),
            filters: [
              {
                text: "Yes",
                value: "Yes",
              },
              {
                text: "No",
                value: "No",
              },
            ],
            onFilter: (value: boolean | React.Key, record: Transaction) =>
              (value === "No" && !record.trade_route) ||
              (value === "Yes" && !!record.trade_route),
          },
        ]
      : []),

    ...(reasons?.mining
      ? [
          {
            title: "Mining",
            dataIndex: "mining",
            key: "mining",
            render: (text: string | null) => text || "N/A", // Display "N/A" if null
            sorter: (a: Transaction, b: Transaction) =>
              (a.mining ?? "").localeCompare(b.mining ?? ""),
            filters: [
              {
                text: "Yes",
                value: "Yes",
              },
              {
                text: "No",
                value: "No",
              },
            ],
            onFilter: (value: boolean | React.Key, record: Transaction) =>
              (value === "No" && !record.mining) ||
              (value === "Yes" && !!record.mining),
          },
        ]
      : []),
  ];
  return (
    <Table
      dataSource={transactions}
      columns={columns}
      pagination={{
        showSizeChanger: true,
        pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
        defaultPageSize: 100,
      }}
    />
  );
}

export default TransactionTable;
