import { useQuery } from "@apollo/client/react";
import { Button, Divider, Flex, Result, Space, Switch, Table } from "antd";
import { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import { GetPossibleTradesQuery, TradeSymbol } from "../gql/graphql";
import { GET_POSSIBLE_TRADES } from "../graphql/queries";

type TradeCandidate =
  GetPossibleTradesQuery["tradeRouteCandidates"]["items"][number];

export default function PossibleTrades() {
  const { loading, error, data, refetch } = useQuery(GET_POSSIBLE_TRADES);
  const [hideFuel, setHideFuel] = useState(true);

  const filteredData = useMemo(() => {
    if (!data?.tradeRouteCandidates.items) return [];
    return data.tradeRouteCandidates.items.filter(
      (trade) => !(trade.symbol == TradeSymbol.Fuel && hideFuel),
    );
  }, [data?.tradeRouteCandidates.items, hideFuel]);

  if (error) {
    return (
      <Result
        status="error"
        title="Possible Trades Error"
        subTitle={error.message}
        extra={[
          <Button key="retry" type="primary" onClick={() => refetch()}>
            Try Again
          </Button>,
        ]}
      />
    );
  }

  return (
    <div style={{ padding: "24px 24px" }}>
      <PageTitle title="Possible Trades" />
      <Flex justify="space-between">
        <Space>
          <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
            Possible Trades {filteredData.length}
          </h1>
          <Button onClick={() => refetch()}>Refresh</Button>
        </Space>
        <Space>
          Hide Fuel
          <Switch onChange={setHideFuel} checked={hideFuel} />
        </Space>
      </Flex>
      <Divider />
      <Table
        loading={loading}
        dataSource={filteredData}
        columns={[
          {
            title: "Symbol",
            dataIndex: "symbol",
            key: "symbol",
            render: (symbol: string) => (
              <Link to={`/supplyChain/${symbol}`}>{symbol}</Link>
            ),
            sorter: (a, b) => a.symbol.localeCompare(b.symbol),
            filters: Object.values(TradeSymbol).map((symbol) => ({
              text: symbol,
              value: symbol,
            })),
            onFilter: (value, record) => record.symbol === value,
          },
          {
            title: "Same System",
            dataIndex: "",
            key: "same_system",
            render: (_: unknown, trade: TradeCandidate) =>
              trade.purchase.waypointSymbol.split("-")[1] ===
              trade.sell.waypointSymbol.split("-")[1]
                ? "Yes"
                : "No",
            sorter: (a, b) => {
              const a_same =
                a.purchase.waypointSymbol.split("-")[1] ===
                a.sell.waypointSymbol.split("-")[1];
              const b_same =
                b.purchase.waypointSymbol.split("-")[1] ===
                b.sell.waypointSymbol.split("-")[1];

              return Number(a_same) - Number(b_same);
            },
            filters: [
              { text: "Yes", value: "yes" },
              { text: "No", value: "no" },
            ],
            onFilter: (value, record) =>
              (record.purchase.waypointSymbol.split("-")[1] ===
                record.sell.waypointSymbol.split("-")[1]) ===
              (value === "yes"),
          },
          {
            title: "From",
            dataIndex: "",
            key: "from",
            render: (_: unknown, trade: TradeCandidate) =>
              `${trade.purchase.waypointSymbol} (${trade.purchase.type})`,
            sorter: (a, b) =>
              a.purchase.waypointSymbol.localeCompare(
                b.purchase.waypointSymbol,
              ),
          },
          {
            title: "To",
            dataIndex: "",
            key: "to",
            render: (_: unknown, trade: TradeCandidate) =>
              `${trade.sell.waypointSymbol} (${trade.sell.type})`,
            sorter: (a, b) =>
              a.sell.waypointSymbol.localeCompare(b.sell.waypointSymbol),
          },
          {
            title: "Purchase Price",
            dataIndex: "",
            key: "purchase_price",
            render: (_: unknown, trade: TradeCandidate) => (
              <>
                {trade.purchaseGood ? (
                  <MoneyDisplay amount={trade.purchaseGood.purchasePrice} />
                ) : (
                  "N/A"
                )}
              </>
            ),
            sorter: (a, b) => {
              const a_purchase_price = a.purchaseGood?.purchasePrice ?? 0;
              const b_purchase_price = b.purchaseGood?.purchasePrice ?? 0;
              return a_purchase_price - b_purchase_price;
            },
          },
          {
            title: "Sell Price",
            dataIndex: "",
            key: "sell_price",
            render: (_: unknown, trade: TradeCandidate) => (
              <>
                {trade.sellGood ? (
                  <MoneyDisplay amount={trade.sellGood.sellPrice} />
                ) : (
                  "N/A"
                )}
              </>
            ),
            sorter: (a, b) => {
              const a_sell_price = a.sellGood?.sellPrice ?? 0;
              const b_sell_price = b.sellGood?.sellPrice ?? 0;
              return a_sell_price - b_sell_price;
            },
          },

          {
            title: "Profit",
            dataIndex: "",
            key: "profit",
            render: (_: unknown, trade: TradeCandidate) => {
              const purchase_price = trade.purchaseGood?.purchasePrice ?? 0;
              const sell_price = trade.sellGood?.sellPrice ?? 0;

              const profit = sell_price - purchase_price;

              return (
                <>
                  <MoneyDisplay amount={profit} />
                </>
              );
            },
            sorter: (a, b) => {
              const a_purchase_price = a.purchaseGood?.purchasePrice ?? 0;
              const a_sell_price = a.sellGood?.sellPrice ?? 0;

              const a_profit = a_sell_price - a_purchase_price;

              const b_purchase_price = b.purchaseGood?.purchasePrice ?? 0;
              const b_sell_price = b.sellGood?.sellPrice ?? 0;

              const b_profit = b_sell_price - b_purchase_price;

              return a_profit - b_profit;
            },
          },
        ]}
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
