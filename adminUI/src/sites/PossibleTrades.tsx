import { Space, Table } from "antd";
import { useEffect, useState } from "react";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { PossibleTrade } from "../models/TradeRoute";
import { backendUrl } from "../MyApp";

export default function PossibleTrades() {
  const [possibleTrades, setPossibleTrades] = useState<PossibleTrade[]>([]);

  useEffect(() => {
    fetch(`http://${backendUrl}/insights/trade/possible`)
      .then((response) => response.json())
      .then((data) => {
        console.log("/insights/trade/possible", data);

        setPossibleTrades(data.trades);
      });
  }, []);

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Possible Trades" />
      <Space>
        <h1>Possible Trades {possibleTrades.length}</h1>
      </Space>
      <Table
        dataSource={possibleTrades}
        columns={[
          {
            title: "Symbol",
            dataIndex: "symbol",
            key: "symbol",
            // render: (symbol) => <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>,
            sorter: (a, b) => a.symbol.localeCompare(b.symbol),
          },
          {
            title: "From",
            dataIndex: "",
            key: "from",
            render: (_: unknown, trade: PossibleTrade) =>
              `${trade.purchase.waypoint_symbol} (${trade.purchase.type})`,
          },
          {
            title: "Purchase Price",
            dataIndex: "",
            key: "purchase_price",
            render: (_: unknown, trade: PossibleTrade) => (
              <>
                {trade.purchase_good ? (
                  <MoneyDisplay amount={trade.purchase_good.purchase_price} />
                ) : (
                  "N/A"
                )}
              </>
            ),
          },
          {
            title: "To",
            dataIndex: "",
            key: "to",
            render: (_: unknown, trade: PossibleTrade) =>
              `${trade.sell.waypoint_symbol} (${trade.sell.type})`,
          },
          {
            title: "Sell Price",
            dataIndex: "",
            key: "sell_price",
            render: (_: unknown, trade: PossibleTrade) => (
              <>
                {trade.sell_good ? (
                  <MoneyDisplay amount={trade.sell_good.sell_price} />
                ) : (
                  "N/A"
                )}
              </>
            ),
          },
          {
            title: "Same System",
            dataIndex: "",
            key: "same_system",
            render: (_: unknown, trade: PossibleTrade) =>
              trade.purchase.waypoint_symbol.split("-")[1] ===
              trade.sell.waypoint_symbol.split("-")[1]
                ? "Yes"
                : "No",
            sorter: (a, b) => {
              const a_system = a.purchase.waypoint_symbol.split("-")[1];
              const b_system = b.purchase.waypoint_symbol.split("-")[1];

              return a_system.localeCompare(b_system);
            },
            filters: [
              { text: "Yes", value: "yes" },
              { text: "No", value: "no" },
            ],
            onFilter: (value, record) =>
              (record.purchase.waypoint_symbol.split("-")[1] ===
                record.sell.waypoint_symbol.split("-")[1]) ===
              (value === "yes"),
          },
          {
            title: "Profit",
            dataIndex: "",
            key: "profit",
            render: (_: unknown, trade: PossibleTrade) => {
              const purchase_price = trade.purchase_good?.purchase_price ?? 0;
              const sell_price = trade.sell_good?.sell_price ?? 0;

              const profit = sell_price - purchase_price;

              return (
                <>
                  <MoneyDisplay amount={profit} />
                </>
              );
            },
            sorter: (a, b) => {
              const a_purchase_price = a.purchase_good?.purchase_price ?? 0;
              const a_sell_price = a.sell_good?.sell_price ?? 0;

              const a_profit = a_sell_price - a_purchase_price;

              const b_purchase_price = b.purchase_good?.purchase_price ?? 0;
              const b_sell_price = b.sell_good?.sell_price ?? 0;

              const b_profit = b_sell_price - b_purchase_price;

              return a_profit - b_profit;
            },
          },
        ]}
      />
    </div>
  );
}
