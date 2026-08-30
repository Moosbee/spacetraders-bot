import { useQuery } from "@apollo/client/react";
import {
  Button,
  Col,
  Divider,
  Flex,
  Result,
  Row,
  Space,
  Spin,
  Table,
} from "antd";
import { Link, useParams } from "react-router-dom";
import MoneyDisplay from "../features/MonyDisplay";
import PageTitle from "../features/PageTitle";
import RenderSupplyChainNeeds from "../features/RenderSupplyChainNeeds/RenderSupplyChainNeeds";
import WaypointLink from "../features/WaypointLink";
import {
  ActivityLevel,
  MarketTradeGoodType,
  SupplyLevel,
  TradeSymbol,
} from "../gql/graphql";
import { GET_TRADE_SYMBOL_SUPPLY_CHAIN } from "../graphql/queries";

export default function TradeSymbolInfo() {
  const { tradeSymbol: tradeSymbolString } = useParams();
  // if (!tradeSymbolString || !isTradeSymbol(tradeSymbolString)) return null;
  const tradeSymbol = tradeSymbolString as TradeSymbol;

  const { loading, error, data, refetch } = useQuery(
    GET_TRADE_SYMBOL_SUPPLY_CHAIN,
    {
      variables: { tradeSymbol: tradeSymbol || "" },
    },
  );

  if (error) {
    return (
      <Result
        status="error"
        title="TradeSymbol Error"
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
      <PageTitle title={`TradeSymbol ${tradeSymbol}`} />
      <Space>
        <h1 className="scroll-m-20 text-center text-3xl font-bold tracking-tight text-balance">
          TradeSymbol: {tradeSymbol}
        </h1>
        <Button
          onClick={() => {
            refetch();
          }}
        >
          Reload
        </Button>
      </Space>
      <Divider />
      <Spin spinning={loading}>
        <Row gutter={16}>
          <Col span={8}>
            <h2>Needs</h2>
            <ul>
              {data?.tradeSymbolInfo.requires.items.map((need) => (
                <li key={need.symbol}>
                  <Link to={`/supplyChain/${need.symbol}`}>{need.symbol}</Link>
                </li>
              ))}
            </ul>
          </Col>
          <Col span={8}>
            <div className="flex flex-col justify-center align-center">
              <h2>Total Supply Chain</h2>
              {RenderSupplyChainNeeds(
                {
                  symbol: tradeSymbol,
                  requires: {
                    items: [
                      {
                        symbol: tradeSymbol,
                        requires: data?.tradeSymbolInfo.requires,
                      },
                    ],
                  },
                },
                true,
              )}
            </div>
          </Col>
          <Col span={8}>
            <h2>Needed by</h2>

            <ul>
              {data?.tradeSymbolInfo.requiredBy.items.map((need) => (
                <li key={need.symbol}>
                  {" "}
                  <Link to={`/supplyChain/${need.symbol}`}>{need.symbol}</Link>
                </li>
              ))}
            </ul>
          </Col>
        </Row>
      </Spin>
      <Divider>Markets</Divider>
      <Table
        size="small"
        title={() => (
          <Flex justify="space-between">
            <span>Market Trades</span>
          </Flex>
        )}
        columns={[
          {
            title: "Waypoint",
            dataIndex: "waypointSymbol",
            key: "waypointSymbol",
            render: (symbol: string) => (
              <WaypointLink waypoint={symbol}>{symbol}</WaypointLink>
            ),
            sorter: (a, b) => a.waypointSymbol.localeCompare(b.waypointSymbol),
            filters: [
              ...new Set(
                (data?.marketTrades.items || []).map((t) => t.waypointSymbol),
              ),
            ].map((t) => ({
              text: t,
              value: t,
            })),
            onFilter: (value, record) => record.waypointSymbol === value,
          },
          {
            title: "Symbol",
            dataIndex: "symbol",
            key: "symbol",
            sorter: (a, b) => a.symbol.localeCompare(b.symbol),
          },
          {
            title: "Type",
            dataIndex: "type",
            key: "type",
            sorter: (a, b) => a.type.localeCompare(b.type),
            filters: Object.values(MarketTradeGoodType).map((t) => ({
              text: t,
              value: t,
            })),
            onFilter: (value, record) => record.type === value,
          },

          {
            title: "Supply",
            key: "marketTradeGood?.supply",
            render: (_, record) => record.marketTradeGood?.supply || "N/A",
            sorter: (a, b) =>
              (a.marketTradeGood?.supply ?? "").localeCompare(
                b.marketTradeGood?.supply ?? "",
              ),
            filters: Object.values(SupplyLevel).map((t) => ({
              text: t,
              value: t,
            })),
            onFilter: (value, record) =>
              record.marketTradeGood?.supply === value,
          },
          {
            title: "Volume",
            key: "marketTradeGood?.tradeVolume",
            align: "right",
            render: (_, record) => record.marketTradeGood?.tradeVolume || "N/A",

            sorter: (a, b) =>
              (a.marketTradeGood?.tradeVolume ?? 0) -
              (b.marketTradeGood?.tradeVolume ?? 0),
          },
          {
            title: "Activity",
            key: "activity",
            render: (_, record) => record.marketTradeGood?.activity || "N/A",
            sorter: (a, b) =>
              (a.marketTradeGood?.activity ?? "").localeCompare(
                b.marketTradeGood?.activity ?? "",
              ),
            filters: Object.values(ActivityLevel).map((t) => ({
              text: t,
              value: t,
            })),
            onFilter: (value, record) =>
              record.marketTradeGood?.activity === value,
          },
          {
            title: "Purchase",
            key: "marketTradeGood?.purchasePrice",
            render: (_, record) =>
              record.type === "IMPORT" ? (
                record.marketTradeGood?.purchasePrice ? (
                  <MoneyDisplay
                    amount={record.marketTradeGood?.purchasePrice}
                  />
                ) : (
                  "N/A"
                )
              ) : (
                <b>
                  {record.marketTradeGood?.purchasePrice ? (
                    <MoneyDisplay
                      amount={record.marketTradeGood?.purchasePrice}
                    />
                  ) : (
                    "N/A"
                  )}
                </b>
              ),
            sorter: (a, b) =>
              (a.marketTradeGood?.purchasePrice || 0) -
              (b.marketTradeGood?.purchasePrice || 0),
          },
          {
            title: "Sell",
            key: "marketTradeGood?.sellPrice",
            render: (_, record) =>
              record.type === "EXPORT" ? (
                record.marketTradeGood?.sellPrice ? (
                  <MoneyDisplay amount={record.marketTradeGood?.sellPrice} />
                ) : (
                  "N/A"
                )
              ) : (
                <b>
                  {record.marketTradeGood?.sellPrice ? (
                    <MoneyDisplay amount={record.marketTradeGood?.sellPrice} />
                  ) : (
                    "N/A"
                  )}
                </b>
              ),
            sorter: (a, b) =>
              (a.marketTradeGood?.sellPrice || 0) -
              (b.marketTradeGood?.sellPrice || 0),
          },
          {
            title: "Created At",
            dataIndex: "createdAt",
            key: "createdAt",
            render: (date: string) => new Date(date).toLocaleString(),
            sorter: (a, b) =>
              new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
          },
        ]}
        dataSource={data?.marketTrades.items || []}
        rowKey={(row) => row.symbol + row.waypointSymbol + row.type}
        pagination={{
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
      <Divider />
      {/* <MarketTransactionTable transactions={[]} /> */}
    </div>
  );
}
