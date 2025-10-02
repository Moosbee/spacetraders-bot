import { Button, Space, Table, TableProps } from "antd";
import { useEffect, useState } from "react";
import { backendUrl } from "../data";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import WaypointLink from "../features/WaypointLink";
import { TradeRoute } from "../models/TradeRoute";

function TradeRoutes() {
  const [tradeRoutes, setTradeRoutes] = useState<TradeRoute[] | null>(null);

  useEffect(() => {
    fetch(`http://${backendUrl}/tradeRoutes`)
      .then((response) => response.json())
      .then((data) => {
        console.log("tradeRoutes", data);

        setTradeRoutes(data);
      });
  }, []);

  const columns: TableProps<TradeRoute>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      sorter: (a, b) => a.id - b.id,
      defaultSortOrder: "descend",
    },
    {
      title: "Trade Symbol",
      dataIndex: "symbol",
      key: "symbol",
      sorter: (a, b) => a.symbol.localeCompare(b.symbol),
    },
    {
      title: "Ship Symbol",
      dataIndex: "ship_symbol",
      key: "ship_symbol",
      sorter: (a, b) => a.ship_symbol.localeCompare(b.ship_symbol),
    },
    {
      title: "Purchase Waypoint",
      dataIndex: "purchase_waypoint",
      key: "purchase_waypoint",
      sorter: (a, b) => a.purchase_waypoint.localeCompare(b.purchase_waypoint),
      render: (value) => <WaypointLink waypoint={value}>{value}</WaypointLink>,
    },
    {
      title: "Sell Waypoint",
      dataIndex: "sell_waypoint",
      key: "sell_waypoint",
      sorter: (a, b) => a.sell_waypoint.localeCompare(b.sell_waypoint),
      render: (value) => <WaypointLink waypoint={value}>{value}</WaypointLink>,
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
      render: (value) => value,
      sorter: (a, b) => (a.status === b.status ? 0 : a.status ? -1 : 1),
      filters: [
        { text: "Delivered", value: "Delivered" },
        { text: "InTransit", value: "InTransit" },
        { text: "Failed", value: "Failed" },
      ],
      onFilter: (value, record) => record.status === value,
    },
    {
      title: "Trade Volume",
      dataIndex: "trade_volume",
      key: "trade_volume",
      sorter: (a, b) => a.trade_volume - b.trade_volume,
      align: "right",
    },
    {
      title: "Reserved Fund",
      dataIndex: "reserved_fund",
      key: "reserved_fund",
      align: "right",
      render: (value) => value,
      sorter: (a, b) => (a.reserved_fund || 0) - (b.reserved_fund || 0),
    },
    {
      title: "Predicted Purchase Price",
      dataIndex: "predicted_purchase_price",
      key: "predicted_purchase_price",
      render: (value, record) => (
        <MoneyDisplay amount={value * record.trade_volume} />
      ),
      align: "right",
      sorter: (a, b) => a.predicted_purchase_price - b.predicted_purchase_price,
    },
    {
      title: "Predicted Sell Price",
      dataIndex: "predicted_sell_price",
      key: "predicted_sell_price",
      render: (value, record) => (
        <MoneyDisplay amount={value * record.trade_volume} />
      ),
      align: "right",
      sorter: (a, b) => a.predicted_sell_price - b.predicted_sell_price,
    },
    // {
    //   title: "Sum",
    //   dataIndex: "sum",
    //   key: "sum",
    //   render: (value) => <MoneyDisplay amount={value} />,
    //   align: "right",
    //   sorter: (a, b) => a.sum - b.sum,
    // },
    {
      title: "Predicted Profit",
      dataIndex: "",
      key: "predicted_profit",
      render: (_, record) => (
        <MoneyDisplay
          amount={
            record.predicted_sell_price * record.trade_volume -
            record.predicted_purchase_price * record.trade_volume
          }
        />
      ),
      align: "right",
      sorter: (a, b) =>
        a.predicted_sell_price * a.trade_volume -
        a.predicted_purchase_price * a.trade_volume -
        (b.predicted_sell_price * b.trade_volume -
          b.predicted_purchase_price * b.trade_volume),
    },
    {
      title: "Delta",
      dataIndex: "",
      key: "delta",
      render: (_, record) => {
        const predicted_purchase_price =
          record.predicted_purchase_price * record.trade_volume;
        const predicted_sell_price =
          record.predicted_sell_price * record.trade_volume;

        const predicted_profit =
          predicted_sell_price - predicted_purchase_price;
        const actual_profit = record.profit;

        const delta = actual_profit - predicted_profit;

        return (
          <MoneyDisplay
            amount={delta}
            // style={{ color: delta < 0 ? "red" : "currentColor" }}
          />
        );
      },
      align: "right",
      sorter: (a, b) => {
        const a_predicted_purchase_price =
          a.predicted_purchase_price * a.trade_volume;
        const a_predicted_sell_price = a.predicted_sell_price * a.trade_volume;

        const a_predicted_profit =
          a_predicted_sell_price - a_predicted_purchase_price;
        const a_actual_profit = a.profit;

        const a_delta = a_actual_profit - a_predicted_profit;

        const b_predicted_purchase_price =
          b.predicted_purchase_price * b.trade_volume;
        const b_predicted_sell_price = b.predicted_sell_price * b.trade_volume;

        const b_predicted_profit =
          b_predicted_sell_price - b_predicted_purchase_price;
        const b_actual_profit = b.profit;

        const b_delta = b_actual_profit - b_predicted_profit;

        return a_delta - b_delta;
      },
    },
    {
      title: "Expenses",
      dataIndex: "expenses",
      key: "expenses",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.expenses - b.expenses,
    },
    {
      title: "Income",
      dataIndex: "income",
      key: "income",
      render: (value) => <MoneyDisplay amount={value} />,
      align: "right",
      sorter: (a, b) => a.income - b.income,
    },
    {
      title: "Profit",
      dataIndex: "profit",
      key: "profit",
      render: (value) => (
        <MoneyDisplay
          amount={value}
          style={{ color: value < 0 ? "red" : "currentColor" }}
        />
      ),
      align: "right",
      sorter: (a, b) => a.profit - b.profit,
    },
  ];

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="TradeRoutes" />
      <Space>
        <h1>TradeRoutes</h1>
        <Button
          onClick={() => {
            fetch(`http://${backendUrl}/tradeRoutes`)
              .then((response) => response.json())
              .then((data) => {
                console.log("Contract", data);

                setTradeRoutes(data);
              });
          }}
        >
          Refresh
        </Button>
      </Space>
      <Table
        dataSource={tradeRoutes || []}
        columns={columns}
        rowKey="id"
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
