import { Table, TableProps } from "antd";
import { Link } from "react-router-dom";
import {
  MarketTransaction,
  MarketTransactionType,
  TradeSymbol,
} from "../../gql/graphql";
import MoneyDisplay from "../MonyDisplay";
import WaypointLink from "../WaypointLink";

function TransactionTable({
  transactions,
  size,
  reasons = {
    contract: true,
    trade_route_id: true,
    mining: true,
    construction_shipment_id: true,
  },
}: {
  size?: TableProps["size"];
  transactions: Partial<MarketTransaction>[];
  reasons?: {
    contract: boolean;
    trade_route_id: boolean;
    mining: boolean;
    construction_shipment_id: boolean;
  };
}) {
  const columns: TableProps<Partial<MarketTransaction>>["columns"] = [
    {
      title: "Waypoint",
      dataIndex: "waypointSymbol",
      key: "waypointSymbol",
      render: (symbol: string | undefined) =>
        symbol ? (
          <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
        ) : (
          "N/A"
        ),
      sorter: (a, b) =>
        (a.waypointSymbol || "").localeCompare(b.waypointSymbol || ""),

      filters: [
        ...new Set(transactions.map((t) => t.waypointSymbol || "")),
      ].map((t) => ({
        text: t,
        value: t,
      })),
      onFilter: (value, record) => record.waypointSymbol === value,
    },
    {
      title: "Ship",
      dataIndex: "shipSymbol",
      key: "shipSymbol",
      render: (symbol: string) => <Link to={`/ships/${symbol}`}>{symbol}</Link>,
      sorter: (a, b) => (a.shipSymbol || "").localeCompare(b.shipSymbol || ""),
      filters: [...new Set(transactions.map((t) => t.shipSymbol || ""))].map(
        (t) => ({
          text: t,
          value: t,
        })
      ),
      onFilter: (value, record) => record.shipSymbol === value,
    },
    {
      title: "Trade Symbol",
      dataIndex: "tradeSymbol",
      key: "tradeSymbol",
      sorter: (a, b) =>
        (a.tradeSymbol || "").localeCompare(b.tradeSymbol || ""),
      filters: Object.values(TradeSymbol)
        .sort((a, b) => a.localeCompare(b))
        .map((type) => ({
          text: type,
          value: type,
        })),
      onFilter: (value, record) => record.tradeSymbol === value,
    },
    {
      title: "Transaction Type",
      dataIndex: "type",
      key: "type",
      sorter: (a, b) => (a.type || "").localeCompare(b.type || ""),
      filters: Object.values(MarketTransactionType).map((type) => ({
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
      sorter: (a, b) => (a.units ?? 0) - (b.units ?? 0),
    },
    {
      title: "Price Per Unit",
      dataIndex: "pricePerUnit",
      key: "pricePerUnit",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => (a.pricePerUnit ?? 0) - (b.pricePerUnit ?? 0),
    },
    {
      title: "Total Price",
      dataIndex: "totalPrice",
      key: "totalPrice",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => (a.totalPrice ?? 0) - (b.totalPrice ?? 0),
    },
    {
      title: "Timestamp",
      dataIndex: "timestamp",
      key: "timestamp",
      render: (value) => new Date(value).toLocaleString(),
      align: "right",
      sorter: (a, b) =>
        new Date(a.timestamp ?? 0).getTime() -
        new Date(b.timestamp ?? 0).getTime(),
      defaultSortOrder: "descend",
    },

    ...(reasons?.contract
      ? [
          {
            title: "Contract",
            dataIndex: "contract_id",
            key: "contract_id",
            render: (text: string | null) => text || "N/A", // Display "N/A" if null
            sorter: (
              a: Partial<MarketTransaction>,
              b: Partial<MarketTransaction>
            ) => (a.contract_id ?? "").localeCompare(b.contract_id ?? ""),
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
            onFilter: (
              value: boolean | React.Key,
              record: Partial<MarketTransaction>
            ) =>
              (value === "No" && !record.contract_id) ||
              (value === "Yes" && !!record.contract_id),
          },
        ]
      : []),
    ...(reasons?.trade_route_id
      ? [
          {
            title: "Trade Route",
            dataIndex: "trade_route_id",
            key: "trade_route_id",
            render: (value: number | null) => (value !== null ? value : "N/A"), // Display "N/A" if null
            sorter: (
              a: Partial<MarketTransaction>,
              b: Partial<MarketTransaction>
            ) => (a.trade_route_id ?? 0) - (b.trade_route_id ?? 0),
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
            onFilter: (
              value: boolean | React.Key,
              record: Partial<MarketTransaction>
            ) =>
              (value === "No" && !record.trade_route_id) ||
              (value === "Yes" && !!record.trade_route_id),
          },
        ]
      : []),

    ...(reasons?.mining
      ? [
          {
            title: "Mining",
            dataIndex: "mining_waypoint_symbol",
            key: "mining_waypoint_symbol",
            render: (text: string | null) => text || "N/A", // Display "N/A" if null
            sorter: (
              a: Partial<MarketTransaction>,
              b: Partial<MarketTransaction>
            ) =>
              (a.mining_waypoint_symbol ?? "").localeCompare(
                b.mining_waypoint_symbol ?? ""
              ),
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
            onFilter: (
              value: boolean | React.Key,
              record: Partial<MarketTransaction>
            ) =>
              (value === "No" && !record.mining_waypoint_symbol) ||
              (value === "Yes" && !!record.mining_waypoint_symbol),
          },
        ]
      : []),
    ...(reasons?.construction_shipment_id
      ? [
          {
            title: "Construction Shipment",
            dataIndex: "construction_shipment_id",
            key: "construction_shipment_id",
            render: (value: number | null) => (value !== null ? value : "N/A"), // Display "N/A" if null
            sorter: (
              a: Partial<MarketTransaction>,
              b: Partial<MarketTransaction>
            ) =>
              (a.construction_shipment_id ?? 0) -
              (b.construction_shipment_id ?? 0),
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
            onFilter: (
              value: boolean | React.Key,
              record: Partial<MarketTransaction>
            ) =>
              (value === "No" && !record.construction_shipment_id) ||
              (value === "Yes" && !!record.construction_shipment_id),
          },
        ]
      : []),
  ];
  return (
    <Table
      rowKey={(id) => "L" + id.id}
      size={size}
      dataSource={transactions}
      columns={columns}
      pagination={{
        showSizeChanger: true,
        pageSizeOptions: ["10", "20", "50", "100", "200", "500", "1000"],
        defaultPageSize: 10,
        showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
      }}
    />
  );
}

export default TransactionTable;
